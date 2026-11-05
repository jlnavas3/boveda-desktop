/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Crea o actualiza una categoría dentro de la bóveda.

use crate::models::{Categoria, ContenidoBoveda};

pub fn guardar_o_crear_categoria(
    boveda: &mut ContenidoBoveda,
    id: &str,
    nombre: &str,
    icono: &str,
) -> Option<String> {
    let nombre_limpio = nombre.trim();
    if nombre_limpio.is_empty() {
        return None;
    }

    let ahora = chrono::Utc::now().timestamp_millis();
    let icono_final = if icono.trim().is_empty() {
        "📁".to_string()
    } else {
        icono.trim().to_string()
    };

    if !id.trim().is_empty() {
        if let Some(cat) = boveda.categorias.iter_mut().find(|c| c.id == id.trim()) {
            cat.nombre = nombre_limpio.to_string();
            cat.icono = icono_final;
            cat.modificada_en = ahora;
            return Some(cat.id.clone());
        }
    }

    let nuevo_id = uuid::Uuid::new_v4().to_string();
    boveda.categorias.push(Categoria {
        id: nuevo_id.clone(),
        nombre: nombre_limpio.to_string(),
        icono: icono_final,
        color_hex: None,
        color: None,
        creada_en: ahora,
        modificada_en: ahora,
    });

    Some(nuevo_id)
}
