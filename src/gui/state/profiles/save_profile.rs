/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Crea o actualiza un Perfil / Identidad dentro de la bóveda.

use crate::models::{ContenidoBoveda, Perfil};

#[allow(clippy::too_many_arguments)]
pub fn guardar_o_crear_perfil(
    boveda: &mut ContenidoBoveda,
    id: &str,
    nombre: &str,
    correo: &str,
    correos_secundarios: &str,
    tipo_documento: &str,
    documento: &str,
    preferencia: &str,
) -> Option<String> {
    let nombre_limpio = nombre.trim();
    if nombre_limpio.is_empty() {
        return None;
    }

    let ahora = chrono::Utc::now().timestamp_millis();
    let doc_opt = if documento.trim().is_empty() {
        None
    } else {
        Some(documento.trim().to_string())
    };

    let tipo_doc_opt = if tipo_documento.trim().is_empty() {
        Some("CI".to_string())
    } else {
        Some(tipo_documento.trim().to_string())
    };

    let lista_secundarios: Vec<String> = correos_secundarios
        .split([',', '\n', ';'])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && s.contains('@'))
        .collect();

    let pref = if preferencia.trim().eq_ignore_ascii_case("documento") {
        "documento".to_string()
    } else {
        "correo".to_string()
    };

    if !id.trim().is_empty() {
        if let Some(p) = boveda.perfiles.iter_mut().find(|p| p.id == id.trim()) {
            p.nombre = nombre_limpio.to_string();
            p.correo_principal = correo.trim().to_string();
            p.correos_secundarios = lista_secundarios;
            p.tipo_documento = tipo_doc_opt;
            p.numero_documento = doc_opt;
            p.preferencia_identificador = pref;
            p.modificada_en = ahora;
            return Some(p.id.clone());
        }
    }

    let nuevo_id = uuid::Uuid::new_v4().to_string();
    boveda.perfiles.push(Perfil {
        id: nuevo_id.clone(),
        nombre: nombre_limpio.to_string(),
        correo_principal: correo.trim().to_string(),
        correos_secundarios: lista_secundarios,
        tipo_documento: tipo_doc_opt,
        numero_documento: doc_opt,
        preferencia_identificador: pref,
        color_hex: None,
        color: None,
        icono: "👤".to_string(),
        tarjetas_ids: Vec::new(),
        tarjeta_predeterminada_id: None,
        cuentas_ids: Vec::new(),
        cuenta_predeterminada_id: None,
        contactos_ids: Vec::new(),
        contacto_predeterminado_id: None,
        documentos_ids: Vec::new(),
        documento_predeterminado_id: None,
        creada_en: ahora,
        modificada_en: ahora,
    });

    Some(nuevo_id)
}
