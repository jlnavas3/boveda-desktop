/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Elimina un Perfil / Identidad de la bóveda y desvincula las entradas que lo referenciaban.

use crate::models::ContenidoBoveda;

pub fn eliminar_perfil_boveda(boveda: &mut ContenidoBoveda, id: &str) -> bool {
    let id_limpio = id.trim();
    if id_limpio.is_empty() {
        return false;
    }

    let inicial = boveda.perfiles.len();
    boveda.perfiles.retain(|p| p.id != id_limpio);

    if boveda.perfiles.len() < inicial {
        for e in &mut boveda.entradas {
            if e.perfil_id.as_deref() == Some(id_limpio) {
                e.perfil_id = None;
            }
        }
        true
    } else {
        false
    }
}
