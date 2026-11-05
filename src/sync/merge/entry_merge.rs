/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Fusión aditiva y sin pérdida de dos o más entradas colisionadas o duplicadas.
//! Réplica fiel de FusionadorEntradas.kt de Bóveda Local Android.

use crate::models::{
    CambioContrasena, DatosPasskey, Entrada, TipoEntrada, TotpSecundario,
};
use crate::sync::merge::passkey_dedup::{consolidar_passkey, son_misma_passkey};
use crate::sync::merge::submodel_merge::fusionar_submodelos;
use chrono::Utc;

/// Fusiona las entradas secundarias en la principal sin destruir datos:
/// - Preserva la contraseña más relevante y respalda contraseñas alternativas en `historial_contrasenas`.
/// - Conserva TOTP principal y factores 2FA secundarios en `totps_secundarios`.
/// - Consolida Passkeys principales y almacena alternativas en `passkeys_secundarias`.
/// - Enlaza submodelos nativos y genera notas de respaldo en caso de conflicto.
/// - Une URLs, etiquetas, colecciones y campos personalizados sin duplicación.
pub fn fusionar_entradas(principal: &Entrada, secundarias: &[Entrada]) -> Entrada {
    // 1. URLs unificadas y deduplicadas
    let mut urls_combinadas = principal.urls.clone();
    for sec in secundarias {
        for url in &sec.urls {
            let u_trim = url.trim();
            if !u_trim.is_empty() && !urls_combinadas.iter().any(|u| u.trim().eq_ignore_ascii_case(u_trim)) {
                urls_combinadas.push(u_trim.to_string());
            }
        }
    }

    // 2. Contraseña y recuperación de alternativas en historial
    let mut contrasena_final = principal.contrasena.clone();
    if contrasena_final.trim().is_empty() {
        if let Some(primera) = secundarias.iter().find(|s| !s.contrasena.trim().is_empty()) {
            contrasena_final = primera.contrasena.clone();
        }
    }

    let mut nuevo_historial = principal.historial_contrasenas.clone();
    let mut claves_alternativas_respaldadas = 0;

    for sec in secundarias {
        for h in &sec.historial_contrasenas {
            if !nuevo_historial.iter().any(|x| x.contrasena == h.contrasena) && h.contrasena != contrasena_final {
                nuevo_historial.push(h.clone());
            }
        }

        if !sec.contrasena.trim().is_empty()
            && sec.contrasena != contrasena_final
            && !nuevo_historial.iter().any(|h| h.contrasena == sec.contrasena)
        {
            let momento = if sec.modificada_en > 0 { sec.modificada_en } else { Utc::now().timestamp_millis() };
            nuevo_historial.push(CambioContrasena {
                contrasena: sec.contrasena.clone(),
                cambiada_en: momento,
            });
            claves_alternativas_respaldadas += 1;
        }
    }

    // 3. TOTP primario y secundarios
    let mut totp_final = principal.secreto_totp.clone();
    let mut totp_emisor_final = principal.totp_emisor.clone();
    let mut totp_digitos_final = principal.totp_digitos;
    let mut totp_periodo_final = principal.totp_periodo;
    let mut totp_algoritmo_final = principal.totp_algoritmo.clone();

    if totp_final.as_deref().unwrap_or("").trim().is_empty() {
        if let Some(sec_con_totp) = secundarias.iter().find(|s| s.secreto_totp.as_deref().is_some_and(|sec| !sec.trim().is_empty())) {
            totp_final = sec_con_totp.secreto_totp.clone();
            totp_emisor_final = if !sec_con_totp.totp_emisor.is_empty() { sec_con_totp.totp_emisor.clone() } else { sec_con_totp.titulo.clone() };
            totp_digitos_final = sec_con_totp.totp_digitos;
            totp_periodo_final = sec_con_totp.totp_periodo;
            totp_algoritmo_final = sec_con_totp.totp_algoritmo.clone();
        }
    }

    let mut totps_secundarios_finales = principal.totps_secundarios.clone();
    for sec in secundarias {
        for t_sec in &sec.totps_secundarios {
            if !totp_final.as_deref().is_some_and(|f| f.trim().eq_ignore_ascii_case(t_sec.secreto.trim()))
                && !totps_secundarios_finales.iter().any(|t| t.secreto.trim().eq_ignore_ascii_case(t_sec.secreto.trim()))
            {
                totps_secundarios_finales.push(t_sec.clone());
            }
        }

        if let Some(sec_secreto) = &sec.secreto_totp {
            let sec_trimmed = sec_secreto.trim();
            if !sec_trimmed.is_empty()
                && !totp_final.as_deref().is_some_and(|f| f.trim().eq_ignore_ascii_case(sec_trimmed))
                && !totps_secundarios_finales.iter().any(|t| t.secreto.trim().eq_ignore_ascii_case(sec_trimmed))
            {
                let momento = if sec.modificada_en > 0 { sec.modificada_en } else { Utc::now().timestamp_millis() };
                totps_secundarios_finales.push(TotpSecundario {
                    id: format!("totp-{}", Utc::now().timestamp_nanos_opt().unwrap_or(0)),
                    secreto: sec_trimmed.to_string(),
                    emisor: if !sec.totp_emisor.is_empty() { sec.totp_emisor.clone() } else { sec.titulo.clone() },
                    digitos: sec.totp_digitos,
                    periodo: sec.totp_periodo,
                    algoritmo: sec.totp_algoritmo.clone(),
                    agregado_en: momento,
                });
            }
        }
    }

    // 4. Passkeys
    let mut passkey_final = principal.passkey.clone();
    let mut passkeys_secundarias_finales = principal.passkeys_secundarias.clone();

    for sec in secundarias {
        let mut candidatas: Vec<DatosPasskey> = Vec::new();
        if let Some(pk) = &sec.passkey {
            candidatas.push(pk.clone());
        }
        candidatas.extend(sec.passkeys_secundarias.clone());

        for cand in candidatas {
            if passkey_final.is_none() {
                passkey_final = Some(cand);
            } else if son_misma_passkey(passkey_final.as_ref(), Some(&cand)) {
                passkey_final = passkey_final.map(|p| consolidar_passkey(&p, &cand));
            } else {
                let pos = passkeys_secundarias_finales.iter().position(|p| son_misma_passkey(Some(p), Some(&cand)));
                if let Some(idx) = pos {
                    passkeys_secundarias_finales[idx] = consolidar_passkey(&passkeys_secundarias_finales[idx], &cand);
                } else {
                    passkeys_secundarias_finales.push(cand);
                }
            }
        }
    }

    // 5. Notas y Submodelos Nativos
    let mut notas_buffer = principal.notas.trim().to_string();
    for sec in secundarias {
        let n = sec.notas.trim();
        if !n.is_empty() && !notas_buffer.contains(n) {
            if !notas_buffer.is_empty() {
                notas_buffer.push_str("\n\n---\n");
            }
            notas_buffer.push_str(n);
        }
    }

    if claves_alternativas_respaldadas > 0 {
        if !notas_buffer.is_empty() {
            notas_buffer.push_str("\n\n---\n");
        }
        notas_buffer.push_str(&format!(
            "[{} clave(s) alternativa(s) respaldada(s) en el historial de contraseñas anteriores al unificar].",
            claves_alternativas_respaldadas
        ));
    }

    let fusion_nativa = fusionar_submodelos(principal, secundarias);
    for nota_sub in fusion_nativa.notas_de_respaldo {
        if !notas_buffer.is_empty() {
            notas_buffer.push_str("\n\n---\n");
        }
        notas_buffer.push_str(&nota_sub);
    }

    // 6. Campos personalizados (deduplicados por etiqueta)
    let mut campos_combinados = principal.campos_personalizados.clone();
    for sec in secundarias {
        for c in &sec.campos_personalizados {
            if !campos_combinados.iter().any(|x| x.etiqueta.trim().eq_ignore_ascii_case(c.etiqueta.trim())) {
                campos_combinados.push(c.clone());
            }
        }
    }

    // 7. Etiquetas
    let mut etiquetas_combinadas = principal.etiquetas.clone();
    for sec in secundarias {
        for et in &sec.etiquetas {
            let et_trim = et.trim();
            if !et_trim.is_empty() && !etiquetas_combinadas.iter().any(|e| e.trim().eq_ignore_ascii_case(et_trim)) {
                etiquetas_combinadas.push(et_trim.to_string());
            }
        }
    }

    // 8. Categorías vinculadas
    let mut categorias_combinadas = principal.categorias.clone();
    for sec in secundarias {
        for cat in &sec.categorias {
            if !categorias_combinadas.contains(cat) {
                categorias_combinadas.push(cat.clone());
            }
        }
    }

    // 9. Tipo de entrada final
    let tipo_final = if principal.tipo != TipoEntrada::Login {
        principal.tipo
    } else if !contrasena_final.is_empty() {
        TipoEntrada::Login
    } else {
        principal.tipo
    };

    Entrada {
        id: principal.id.clone(),
        tipo: tipo_final,
        titulo: principal.titulo.clone(),
        usuario: principal.usuario.clone(),
        contrasena: contrasena_final,
        urls: urls_combinadas,
        notas: notas_buffer,
        secreto_totp: totp_final,
        totp_emisor: totp_emisor_final,
        totp_digitos: totp_digitos_final,
        totp_periodo: totp_periodo_final,
        totp_algoritmo: totp_algoritmo_final,
        favorito: principal.favorito || secundarias.iter().any(|s| s.favorito),
        anclado: principal.anclado || secundarias.iter().any(|s| s.anclado),
        creada_en: if principal.creada_en > 0 { principal.creada_en } else { secundarias.iter().map(|s| s.creada_en).filter(|&t| t > 0).min().unwrap_or(0) },
        modificada_en: Utc::now().timestamp_millis(),
        ultimo_uso_en: principal.ultimo_uso_en.max(secundarias.iter().map(|s| s.ultimo_uso_en).max().unwrap_or(0)),
        eliminada_en: principal.eliminada_en,
        etiquetas: etiquetas_combinadas,
        historial_contrasenas: nuevo_historial,
        passkey: passkey_final,
        passkeys_secundarias: passkeys_secundarias_finales,
        totps_secundarios: totps_secundarios_finales,
        campos_personalizados: campos_combinados,
        datos_tarjeta: fusion_nativa.datos_tarjeta,
        datos_wifi: fusion_nativa.datos_wifi,
        datos_cuenta_bancaria: fusion_nativa.datos_cuenta_bancaria,
        datos_identidad: fusion_nativa.datos_identidad,
        datos_servidor: fusion_nativa.datos_servidor,
        datos_wallet: fusion_nativa.datos_wallet,
        datos_contacto: fusion_nativa.datos_contacto,
        ignorada_en_salud: principal.ignorada_en_salud,
        categorias: categorias_combinadas,
        perfil_id: principal.perfil_id.clone().or_else(|| secundarias.iter().find_map(|s| s.perfil_id.clone())),
        fecha_caducidad: principal.fecha_caducidad.or_else(|| secundarias.iter().find_map(|s| s.fecha_caducidad)),
        adjuntos: principal.adjuntos.clone(),
        titulo_personalizado: principal.titulo_personalizado,
        revision: principal
            .revision
            .max(secundarias.iter().map(|s| s.revision).max().unwrap_or(1)),
        last_sync_hash: principal.last_sync_hash,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fusion_preserva_contrasena_alternativa_en_historial() {
        let p = Entrada {
            id: "1".into(),
            titulo: "Sitio".into(),
            usuario: "admin".into(),
            contrasena: "claveNueva2026".into(),
            modificada_en: 200,
            ..Default::default()
        };

        let s = Entrada {
            id: "1".into(),
            titulo: "Sitio".into(),
            usuario: "admin".into(),
            contrasena: "claveVieja2025".into(),
            modificada_en: 100,
            ..Default::default()
        };

        let fusionada = fusionar_entradas(&p, &[s]);
        assert_eq!(fusionada.contrasena, "claveNueva2026");
        assert_eq!(fusionada.historial_contrasenas.len(), 1);
        assert_eq!(fusionada.historial_contrasenas[0].contrasena, "claveVieja2025");
        assert!(fusionada.notas.contains("clave(s) alternativa(s) respaldada(s)"));
    }

    #[test]
    fn test_fusion_absorbe_totp_secundario() {
        let p = Entrada {
            id: "1".into(),
            titulo: "GitHub".into(),
            secreto_totp: Some("JBSWY3DPEHPK3PXP".into()),
            totp_emisor: "GitHub".into(),
            ..Default::default()
        };

        let s = Entrada {
            id: "1".into(),
            titulo: "GitHub".into(),
            secreto_totp: Some("KRSXG5CTMVRXEZLU".into()), // TOTP diferente
            totp_emisor: "GitHub Secundario".into(),
            ..Default::default()
        };

        let fusionada = fusionar_entradas(&p, &[s]);
        assert_eq!(fusionada.secreto_totp.as_deref(), Some("JBSWY3DPEHPK3PXP"));
        assert_eq!(fusionada.totps_secundarios.len(), 1);
        assert_eq!(fusionada.totps_secundarios[0].secreto, "KRSXG5CTMVRXEZLU");
    }
}
