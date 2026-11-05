/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Normalizador canónico de servicios y usuarios para detección de duplicados.
//! Réplica fiel de `NormalizadorDuplicados.kt` de Android.

use crate::gui::domain::service_extractor::extraer_nombre_servicio_canonico;
use crate::models::Entrada;

/// Normaliza el nombre de servicio para unificación de duplicados (Android: `NormalizadorDuplicados.normalizarServicio`).
pub fn normalizar_servicio_duplicados(entrada: &Entrada) -> String {
    let url = entrada.urls.iter().find(|u| !u.trim().is_empty()).map(|s| s.as_str()).unwrap_or("");
    let base = extraer_nombre_servicio_canonico(url, &entrada.titulo).trim().to_lowercase();

    if !base.is_empty() && base != "cuenta" && base != "general" {
        return base;
    }

    let tit = entrada.titulo.trim().to_lowercase();
    if !tit.is_empty() {
        return tit;
    }

    "desconocido".to_string()
}

/// Normaliza el nombre de usuario según el servicio para detectar variantes de la misma cuenta
/// (Android: `NormalizadorDuplicados.normalizarUsuario`).
pub fn normalizar_usuario_duplicados(usuario: &str, servicio: &str) -> String {
    let u = usuario.trim().to_lowercase();
    if u.is_empty() {
        return String::new();
    }

    let serv_lower = servicio.to_lowercase();

    // 1. Google / Gmail:
    // "@googlemail.com" es el alias histórico idéntico de "@gmail.com"
    if u.ends_with("@googlemail.com") {
        let prefix = u.strip_suffix("@googlemail.com").unwrap_or(&u);
        return format!("{prefix}@gmail.com");
    }
    if (serv_lower.contains("google") || serv_lower.contains("gmail")) && !u.contains('@') {
        return format!("{u}@gmail.com");
    }
    if u.ends_with("@gmail.com") {
        return u;
    }

    // 2. Si el servicio es un dominio propio (ej. en spotify.com usuario es 'juan' y otro 'juan@spotify.com'):
    // Solo unificar si una entrada omitió la arroba y la otra puso @<dominioServicio>
    let dominio_serv = serv_lower
        .trim_start_matches("web:")
        .trim_start_matches("app:");
    if !dominio_serv.is_empty() && dominio_serv.contains('.') && !u.contains('@') {
        return format!("{u}@{dominio_serv}");
    }

    // Normalización de números telefónicos (quitar espacios, guiones, paréntesis)
    let solo_digitos: String = u.chars().filter(|c| c.is_ascii_digit()).collect();
    if solo_digitos.len() >= 7
        && u.chars().all(|c| c.is_ascii_digit() || c == '+' || c == '-' || c == ' ' || c == '(' || c == ')')
    {
        let mut digitos = solo_digitos;
        if digitos.starts_with("593") && digitos.len() == 12 {
            digitos = format!("0{}", &digitos[3..]);
        }
        return digitos;
    }

    u
}

/// Comprueba si la entrada contiene enlaces de esquemas de apps Android.
pub fn es_app_android_entrada(entrada: &Entrada) -> bool {
    entrada.urls.iter().any(|u| u.trim().to_lowercase().starts_with("android://"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalizar_usuario() {
        assert_eq!(normalizar_usuario_duplicados("demo", "google"), "demo@gmail.com");
        assert_eq!(normalizar_usuario_duplicados("demo@googlemail.com", "google"), "demo@gmail.com");
        assert_eq!(normalizar_usuario_duplicados("+593991234567", "whatsapp"), "0991234567");

        // Casos institucionales y de servicios que NO deben cruzarse:
        assert_eq!(normalizar_usuario_duplicados("jlnavas3", "utpl"), "jlnavas3");
        assert_eq!(normalizar_usuario_duplicados("jlnavas3@utpl.edu.ec", "utpl"), "jlnavas3@utpl.edu.ec");
        assert_ne!(
            normalizar_usuario_duplicados("jlnavas3", "utpl"),
            normalizar_usuario_duplicados("jlnavas3@utpl.edu.ec", "utpl")
        );

        let e_learndigital = Entrada {
            titulo: "Learndigital".to_string(),
            urls: vec!["https://learndigital.withgoogle.com/activate/register".to_string()],
            ..Default::default()
        };
        assert_eq!(normalizar_servicio_duplicados(&e_learndigital), "learndigital");
    }
}
