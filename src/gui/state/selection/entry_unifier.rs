/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Unificador sin pérdida (Lossless Merge) de entradas seleccionadas en la bóveda,
//! con paridad funcional exacta respecto a `FusionadorEntradas.kt` de Android.

use std::collections::HashSet;

use crate::models::cambio_contrasena::CambioContrasena;
use crate::models::passkey::DatosPasskey;
use crate::models::totp_secundario::TotpSecundario;
use crate::models::{ContenidoBoveda, Entrada, TipoEntrada};

/// Selecciona la mejor entrada candidata como principal dentro de un conjunto.
pub fn seleccionar_mejor_entrada(entradas: &[Entrada]) -> Option<String> {
    entradas
        .iter()
        .max_by_key(|e| {
            (
                u8::from(e.favorito),
                u8::from(e.tiene_totp()),
                u8::from(e.passkey.is_some()),
                e.campos_personalizados.len(),
                e.notas.len(),
                u8::from(e.usuario.contains('@')),
                e.usuario.len(),
                e.modificada_en,
            )
        })
        .map(|e| e.id.clone())
}

/// Unifica todas las entradas cuyos IDs están en `ids` dentro de la entrada principal,
/// preservando contraseñas anteriores, passkeys secundarias, TOTPs secundarios, URLs,
/// etiquetas, categorías y campos personalizados, y eliminando las secundarias absorbidas.
pub fn unificar_entradas_seleccionadas(
    boveda: &mut ContenidoBoveda,
    ids: &HashSet<String>,
    id_principal_preferido: Option<&str>,
) -> Option<String> {
    let seleccionadas: Vec<Entrada> = boveda
        .entradas
        .iter()
        .filter(|e| ids.contains(&e.id))
        .cloned()
        .collect();

    if seleccionadas.len() < 2 {
        return None;
    }

    let id_principal = id_principal_preferido
        .filter(|id| seleccionadas.iter().any(|e| e.id == *id))
        .map(|s| s.to_string())
        .or_else(|| seleccionar_mejor_entrada(&seleccionadas))?;

    let mut principal = seleccionadas
        .iter()
        .find(|e| e.id == id_principal)?
        .clone();
    let secundarias: Vec<Entrada> = seleccionadas
        .into_iter()
        .filter(|e| e.id != id_principal)
        .collect();

    let ahora = chrono::Utc::now().timestamp_millis();

    // 1. Si la principal no tiene contraseña pero una secundaria sí, adoptarla
    if principal.contrasena.trim().is_empty() {
        if let Some(sec_con_clave) = secundarias.iter().find(|s| !s.contrasena.trim().is_empty()) {
            principal.contrasena = sec_con_clave.contrasena.clone();
        }
    }

    // 2. Si la principal no tiene usuario pero una secundaria sí, adoptarlo
    if principal.usuario.trim().is_empty() {
        if let Some(sec_con_usr) = secundarias.iter().find(|s| !s.usuario.trim().is_empty()) {
            principal.usuario = sec_con_usr.usuario.clone();
        }
    }

    let mut usuarios_alternativos = Vec::new();
    let mut claves_alternativas_guardadas = 0usize;

    for sec in &secundarias {
        // Combinar URLs sin duplicados
        for u in &sec.urls {
            let u_trim = u.trim();
            if !u_trim.is_empty()
                && !principal
                    .urls
                    .iter()
                    .any(|p| p.trim().eq_ignore_ascii_case(u_trim))
            {
                principal.urls.push(u_trim.to_string());
            }
        }

        // Combinar categorías
        for cat in &sec.categorias {
            if !principal.categorias.contains(cat) {
                principal.categorias.push(cat.clone());
            }
        }

        // Combinar etiquetas
        for tag in &sec.etiquetas {
            if !principal
                .etiquetas
                .iter()
                .any(|t| t.eq_ignore_ascii_case(tag))
            {
                principal.etiquetas.push(tag.clone());
            }
        }

        // Combinar notas
        let n = sec.notas.trim();
        if !n.is_empty() && !principal.notas.contains(n) {
            if !principal.notas.trim().is_empty() {
                principal.notas.push_str("\n\n---\n");
            }
            principal.notas.push_str(n);
        }

        // Usuarios alternativos
        let u_sec = sec.usuario.trim();
        if !u_sec.is_empty()
            && !u_sec.eq_ignore_ascii_case(principal.usuario.trim())
            && !usuarios_alternativos.contains(&u_sec.to_string())
        {
            usuarios_alternativos.push(u_sec.to_string());
        }

        // Contraseñas anteriores
        if !sec.contrasena.is_empty() && sec.contrasena != principal.contrasena {
            claves_alternativas_guardadas += 1;
            if !principal
                .historial_contrasenas
                .iter()
                .any(|h| h.contrasena == sec.contrasena)
            {
                let momento = if sec.modificada_en > 0 {
                    sec.modificada_en
                } else {
                    ahora
                };
                principal.historial_contrasenas.push(CambioContrasena {
                    contrasena: sec.contrasena.clone(),
                    cambiada_en: momento,
                });
            }
        }
        for h in &sec.historial_contrasenas {
            if !h.contrasena.is_empty()
                && h.contrasena != principal.contrasena
                && !principal
                    .historial_contrasenas
                    .iter()
                    .any(| ph | ph.contrasena == h.contrasena)
            {
                principal.historial_contrasenas.push(h.clone());
            }
        }

        // TOTP principal y secundarios
        if let Some(ref sec_totp) = sec.secreto_totp {
            let st_trim = sec_totp.trim();
            if !st_trim.is_empty() {
                let prin_totp = principal
                    .secreto_totp
                    .as_deref()
                    .map(|s| s.trim())
                    .unwrap_or("");
                if prin_totp.is_empty() {
                    principal.secreto_totp = Some(st_trim.to_string());
                    if principal.totp_emisor.trim().is_empty() {
                        principal.totp_emisor = if !sec.totp_emisor.trim().is_empty() {
                            sec.totp_emisor.clone()
                        } else {
                            sec.titulo.clone()
                        };
                    }
                    principal.totp_digitos = sec.totp_digitos;
                    principal.totp_periodo = sec.totp_periodo;
                    principal.totp_algoritmo = sec.totp_algoritmo.clone();
                } else if !prin_totp.eq_ignore_ascii_case(st_trim)
                    && !principal
                        .totps_secundarios
                        .iter()
                        .any(|t| t.secreto.trim().eq_ignore_ascii_case(st_trim))
                {
                    principal.totps_secundarios.push(TotpSecundario {
                        id: uuid::Uuid::new_v4().to_string(),
                        secreto: st_trim.to_string(),
                        emisor: if !sec.totp_emisor.trim().is_empty() {
                            sec.totp_emisor.clone()
                        } else {
                            sec.titulo.clone()
                        },
                        periodo: sec.totp_periodo,
                        digitos: sec.totp_digitos,
                        algoritmo: sec.totp_algoritmo.clone(),
                        agregado_en: if sec.modificada_en > 0 {
                            sec.modificada_en
                        } else {
                            ahora
                        },
                    });
                }
            }
        }
        for ts in &sec.totps_secundarios {
            let prin_totp = principal
                .secreto_totp
                .as_deref()
                .map(|s| s.trim())
                .unwrap_or("");
            if !ts.secreto.trim().eq_ignore_ascii_case(prin_totp)
                && !principal
                    .totps_secundarios
                    .iter()
                    .any(|t| t.secreto.trim().eq_ignore_ascii_case(ts.secreto.trim()))
            {
                principal.totps_secundarios.push(ts.clone());
            }
        }

        // Passkey principal y secundarias
        if let Some(ref sec_pk) = sec.passkey {
            if principal.passkey.is_none() {
                principal.passkey = Some(sec_pk.clone());
            } else if let Some(ref prin_pk) = principal.passkey {
                if !son_misma_passkey(prin_pk, sec_pk)
                    && !principal
                        .passkeys_secundarias
                        .iter()
                        .any(|p| son_misma_passkey(p, sec_pk))
                {
                    principal.passkeys_secundarias.push(sec_pk.clone());
                }
            }
        }
        for pks in &sec.passkeys_secundarias {
            let coincide_prin = principal
                .passkey
                .as_ref()
                .map(|p| son_misma_passkey(p, pks))
                .unwrap_or(false);
            if !coincide_prin
                && !principal
                    .passkeys_secundarias
                    .iter()
                    .any(|p| son_misma_passkey(p, pks))
            {
                principal.passkeys_secundarias.push(pks.clone());
            }
        }

        // Campos personalizados
        for cp in &sec.campos_personalizados {
            if !principal
                .campos_personalizados
                .iter()
                .any(|p| p.etiqueta.eq_ignore_ascii_case(&cp.etiqueta))
            {
                principal.campos_personalizados.push(cp.clone());
            }
        }

        // Adjuntos
        for adj in &sec.adjuntos {
            if !principal.adjuntos.iter().any(|a| a.id == adj.id) {
                principal.adjuntos.push(adj.clone());
            }
        }
    }

    if principal.tipo == TipoEntrada::Passkey && !principal.contrasena.trim().is_empty() {
        principal.tipo = TipoEntrada::Login;
    }

    if claves_alternativas_guardadas > 0 {
        if !principal.notas.trim().is_empty() {
            principal.notas.push_str("\n\n---\n");
        }
        principal.notas.push_str(&format!(
            "[{claves_alternativas_guardadas} clave(s) alternativa(s) respaldada(s) en el historial de contraseñas anteriores al unificar]."
        ));
    }

    if !usuarios_alternativos.is_empty() {
        if !principal.notas.trim().is_empty() {
            principal.notas.push_str("\n\n---\n");
        }
        principal.notas.push_str(&format!(
            "Usuario(s) alternativo(s) respaldado(s) al unificar: {}.",
            usuarios_alternativos.join(", ")
        ));
    }

    principal.modificada_en = ahora;

    // Actualizar la entrada principal y remover las secundarias fusionadas
    let ids_secundarios: HashSet<&str> = secundarias.iter().map(|s| s.id.as_str()).collect();
    boveda
        .entradas
        .retain(|e| !ids_secundarios.contains(e.id.as_str()));
    if let Some(slot) = boveda.entradas.iter_mut().find(|e| e.id == id_principal) {
        *slot = principal;
    }

    Some(id_principal)
}

fn son_misma_passkey(p1: &DatosPasskey, p2: &DatosPasskey) -> bool {
    if !p1.cred_id.trim().is_empty()
        && !p2.cred_id.trim().is_empty()
        && p1.cred_id.trim().eq_ignore_ascii_case(p2.cred_id.trim())
    {
        return true;
    }
    if !p1.clave_privada.trim().is_empty()
        && !p2.clave_privada.trim().is_empty()
        && p1.clave_privada == p2.clave_privada
    {
        return true;
    }
    if !p1.rp_id.trim().is_empty()
        && p1.rp_id.trim().eq_ignore_ascii_case(p2.rp_id.trim())
        && !p1.user_handle.trim().is_empty()
        && p1.user_handle == p2.user_handle
    {
        return true;
    }
    false
}
