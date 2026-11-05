/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Elimina una categoría de la bóveda y desvincula las entradas que la referenciaban.

use crate::models::ContenidoBoveda;

pub fn eliminar_categoria_boveda(boveda: &mut ContenidoBoveda, id: &str) -> bool {
    let id_limpio = id.trim();
    if id_limpio.is_empty() {
        return false;
    }

    let inicial = boveda.categorias.len();
    boveda.categorias.retain(|c| c.id != id_limpio);

    if boveda.categorias.len() < inicial {
        for e in &mut boveda.entradas {
            e.categorias.retain(|cid| cid != id_limpio);
        }
        true
    } else {
        false
    }
}
