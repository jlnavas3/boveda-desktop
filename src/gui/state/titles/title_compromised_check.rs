/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Verificador canónico de títulos comprometidos / técnicos y protección manual.
//! Réplica fiel en Rust de `CalculadorAvisosInternos.esTituloComprometido` y
//! `NormalizadorTitulosSitios.esEntradaProtegidaManual` de Android.

use crate::gui::domain::extract_host::extraer_host_limpio;
use crate::gui::state::indicator_detector::es_paquete_app;
use crate::models::Entrada;

/// Lista por defecto de TLDs descartables en Android (AjustesDefaults.NormalizacionTitulos.TLDS_DESCARTABLES)
pub const TLDS_DESCARTABLES_DEFAULT: &[&str] = &[
    "com", "org", "net", "edu", "gov", "mil", "int", "io", "co", "app", "dev", "ai", "tech",
    "online", "site", "xyz", "info", "biz", "me", "tv", "cc", "es", "ec", "mx", "ar", "cl",
    "co", "pe", "uy", "bo", "py", "ve", "cr", "gt", "pa", "hn", "sv", "ni", "do", "pr", "cu",
    "us", "uk", "de", "fr", "it", "pt", "br", "ca", "eu", "ru", "cn", "jp", "kr", "in", "au",
];

/// Evalúa si el título de una credencial es un título técnico, crudo o comprometido
/// que amerita ser normalizado (e.g. URLs crudas, paquetes Android, direcciones IP, puertos o hosts).
pub fn es_titulo_comprometido(entrada: &Entrada, tlds_configurados: &[String]) -> bool {
    let t = entrada.titulo.trim();
    if t.is_empty() {
        return true;
    }

    // Extraer base antes de paréntesis, puntos medios o corchetes
    let t_base = t
        .split('(')
        .next()
        .unwrap_or(t)
        .split('·')
        .next()
        .unwrap_or(t)
        .split('[')
        .next()
        .unwrap_or(t)
        .trim();

    if t_base.is_empty() {
        return true;
    }

    let t_lower = t_base.to_lowercase();
    if t_lower == "localhost" || t_lower.starts_with("localhost :") {
        return false;
    }

    // 1. Esquemas de URLs crudas
    if t_lower.starts_with("http://")
        || t_lower.starts_with("https://")
        || t_lower.starts_with("android://")
        || t_lower.starts_with("androidapp://")
        || t_lower.starts_with("app://")
    {
        return true;
    }

    // 2. Paquetes Android
    if es_paquete_app(t_base) {
        return true;
    }

    // 3. URLs sin espacio con barras o query strings
    if !t_base.contains(' ') && (t_base.contains('/') || t_base.contains('?') || t_base.contains('#')) {
        return true;
    }

    // 4. Puertos sin espacio previo (:8080)
    if let Some(pos) = t_base.find(':') {
        if pos > 0 {
            let despues = &t_base[pos + 1..];
            let digitos: String = despues.chars().take_while(|c| c.is_ascii_digit()).collect();
            if digitos.len() >= 2 && digitos.len() <= 5 {
                let antes = t_base.as_bytes()[pos - 1];
                if antes != b' ' {
                    return true;
                }
            }
        }
    }

    // 5. Direcciones IP numéricas al inicio (e.g. 192.168.1.1)
    let partes_ip: Vec<&str> = t_base.split('.').collect();
    if partes_ip.len() >= 4 && partes_ip[0].chars().all(|c| c.is_ascii_digit()) {
        if let (Ok(o1), Ok(o2)) = (partes_ip[0].parse::<u16>(), partes_ip[1].parse::<u16>()) {
            if o1 <= 255 && o2 <= 255 {
                return true;
            }
        }
    }

    // 6. Termina en un TLD web (.com, .net, .org, etc.)
    let tlds_a_usar: Vec<&str> = if !tlds_configurados.is_empty() {
        tlds_configurados.iter().map(|s| s.as_str()).collect()
    } else {
        TLDS_DESCARTABLES_DEFAULT.to_vec()
    };

    for tld in tlds_a_usar {
        let sufijo = format!(".{}", tld.trim_start_matches('.').to_lowercase());
        if t_lower.ends_with(&sufijo) {
            return true;
        }
    }

    // 7. Si no contiene espacios y contiene un punto con dominio reconocible
    if !t_base.contains(' ') && t_base.contains('.') {
        let partes: Vec<&str> = t_lower.split('.').collect();
        if partes.len() >= 2 {
            let ult = partes.last().unwrap_or(&"");
            if TLDS_DESCARTABLES_DEFAULT.contains(ult) {
                return true;
            }
        }
    }

    // 8. Coincide exactamente con el host crudo de una de sus URLs
    for u in &entrada.urls {
        let host = extraer_host_limpio(u);
        if !host.is_empty() && host.contains('.') {
            let host_low = host.to_lowercase();
            let host_sin_www = host_low.trim_start_matches("www.");
            if t_lower == host_low || t_lower == host_sin_www {
                return true;
            }
        }
    }

    false
}

/// Determina si una entrada fue personalizada a mano o debe protegerse contra sobrescritura.
pub fn es_entrada_protegida_manual(
    entrada: &Entrada,
    nombre_base: &str,
    tlds_configurados: &[String],
) -> bool {
    // 1. Marca explícita
    if entrada.titulo_personalizado {
        return true;
    }

    // 2. Si fue modificada y el título no es técnico/comprometido
    if entrada.modificada_en > 0
        && entrada.modificada_en > entrada.creada_en
        && !es_titulo_comprometido(entrada, tlds_configurados)
    {
        return true;
    }

    // 3. Si el título no es generado ni técnico
    !es_titulo_generado_o_modificable(
        &entrada.titulo,
        nombre_base,
        &entrada.usuario,
        entrada,
        tlds_configurados,
    )
}

fn es_titulo_generado_o_modificable(
    titulo: &str,
    nombre_base: &str,
    usuario: &str,
    entrada: &Entrada,
    tlds_configurados: &[String],
) -> bool {
    if es_titulo_comprometido(entrada, tlds_configurados) {
        return true;
    }
    let t = titulo.trim().to_lowercase();
    let base = nombre_base.trim().to_lowercase();
    if t == base {
        return true;
    }
    if !usuario.trim().is_empty() {
        let u = usuario.trim().to_lowercase();
        if t == format!("{base} ({u})") || t == format!("{base} · {u}") {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_titulos_limpios_no_son_comprometidos() {
        let e1 = Entrada {
            titulo: "JetBrains Account".into(),
            urls: vec!["https://account.jetbrains.com".into()],
            ..Default::default()
        };
        assert!(!es_titulo_comprometido(&e1, &[]));

        let e2 = Entrada {
            titulo: "Twitter".into(),
            urls: vec!["https://twitter.com".into()],
            ..Default::default()
        };
        assert!(!es_titulo_comprometido(&e2, &[]));

        let e3 = Entrada {
            titulo: "NextDNS".into(),
            urls: vec!["https://nextdns.io".into()],
            ..Default::default()
        };
        assert!(!es_titulo_comprometido(&e3, &[]));
    }

    #[test]
    fn test_titulos_tecnicos_son_comprometidos() {
        let e1 = Entrada {
            titulo: "https://www.amazon.com/login".into(),
            urls: vec!["https://www.amazon.com".into()],
            ..Default::default()
        };
        assert!(es_titulo_comprometido(&e1, &[]));

        let e2 = Entrada {
            titulo: "amazon.com".into(),
            urls: vec!["https://www.amazon.com".into()],
            ..Default::default()
        };
        assert!(es_titulo_comprometido(&e2, &[]));

        let e3 = Entrada {
            titulo: "com.whatsapp".into(),
            urls: vec![],
            ..Default::default()
        };
        assert!(es_titulo_comprometido(&e3, &[]));
    }
}
