/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Operación en lote para alternar el estado `anclado` de las entradas seleccionadas.

use std::collections::HashSet;

use crate::models::Entrada;

/// Si todas las seleccionadas están ancladas, las desancla; en caso contrario, las ancla todas.
/// Devuelve el nuevo estado aplicado (`true` = ancladas, `false` = desancladas).
pub fn alternar_anclado_lote(entradas: &mut [Entrada], ids: &HashSet<String>) -> bool {
    let todos_anclados = entradas
        .iter()
        .filter(|e| ids.contains(&e.id))
        .all(|e| e.anclado);
    let nuevo_estado = !todos_anclados;
    let ahora = chrono::Utc::now().timestamp_millis();

    for e in entradas.iter_mut() {
        if ids.contains(&e.id) {
            e.anclado = nuevo_estado;
            e.modificada_en = ahora;
        }
    }
    nuevo_estado
}
