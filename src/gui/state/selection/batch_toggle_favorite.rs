/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Operación en lote para alternar el estado `favorito` de las entradas seleccionadas.

use std::collections::HashSet;

use crate::models::Entrada;

/// Si todas las seleccionadas son favoritas, les quita el favorito; en caso contrario, las marca todas como favoritas.
/// Devuelve el nuevo estado aplicado (`true` = favoritas, `false` = no favoritas).
pub fn alternar_favorito_lote(entradas: &mut [Entrada], ids: &HashSet<String>) -> bool {
    let todos_favoritos = entradas
        .iter()
        .filter(|e| ids.contains(&e.id))
        .all(|e| e.favorito);
    let nuevo_estado = !todos_favoritos;
    let ahora = chrono::Utc::now().timestamp_millis();

    for e in entradas.iter_mut() {
        if ids.contains(&e.id) {
            e.favorito = nuevo_estado;
            e.modificada_en = ahora;
        }
    }
    nuevo_estado
}
