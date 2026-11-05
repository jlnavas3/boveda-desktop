/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Operación en lote para asignar o quitar una categoría sobre las entradas seleccionadas.

use std::collections::HashSet;

use crate::models::Entrada;

/// Alterna la categoría `cat_id` en el conjunto de entradas seleccionadas.
/// Si `cat_id` es `"SIN_CATEGORIA"`, limpia las categorías de todas las seleccionadas.
pub fn asignar_categorias_lote(
    entradas: &mut [Entrada],
    ids: &HashSet<String>,
    cat_id: &str,
) -> bool {
    let cat_trim = cat_id.trim();
    if cat_trim.is_empty() {
        return false;
    }
    let ahora = chrono::Utc::now().timestamp_millis();

    if cat_trim == "SIN_CATEGORIA" {
        for e in entradas.iter_mut() {
            if ids.contains(&e.id) {
                e.categorias.clear();
                e.modificada_en = ahora;
            }
        }
        return false;
    }

    let todas_tienen = entradas
        .iter()
        .filter(|e| ids.contains(&e.id))
        .all(|e| e.categorias.iter().any(|c| c == cat_trim));

    for e in entradas.iter_mut() {
        if ids.contains(&e.id) {
            if todas_tienen {
                e.categorias.retain(|c| c != cat_trim);
            } else if !e.categorias.iter().any(|c| c == cat_trim) {
                e.categorias.push(cat_trim.to_string());
            }
            e.modificada_en = ahora;
        }
    }

    !todas_tienen
}
