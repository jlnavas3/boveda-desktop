/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Operación en lote para renombrar múltiples entradas seleccionadas.

use std::collections::HashSet;

use crate::models::Entrada;

/// Renombra todas las entradas seleccionadas con `nuevo_titulo` y activa `titulo_personalizado = true`.
pub fn renombrar_lote(entradas: &mut [Entrada], ids: &HashSet<String>, nuevo_titulo: &str) -> usize {
    let limpio = nuevo_titulo.trim();
    if limpio.is_empty() {
        return 0;
    }
    let ahora = chrono::Utc::now().timestamp_millis();
    let mut modificadas = 0usize;

    for e in entradas.iter_mut() {
        if ids.contains(&e.id) {
            e.titulo = limpio.to_string();
            e.titulo_personalizado = true;
            e.modificada_en = ahora;
            modificadas += 1;
        }
    }
    modificadas
}
