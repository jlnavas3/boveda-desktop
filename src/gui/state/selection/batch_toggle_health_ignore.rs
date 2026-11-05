/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Operación en lote para alternar el estado `ignorada_en_salud` de las entradas seleccionadas.

use std::collections::HashSet;

use crate::models::Entrada;

/// Si todas las seleccionadas están ignoradas en auditoría, las incluye; en caso contrario, las ignora todas.
pub fn alternar_ignorar_salud_lote(entradas: &mut [Entrada], ids: &HashSet<String>) -> bool {
    let todos_ignorados = entradas
        .iter()
        .filter(|e| ids.contains(&e.id))
        .all(|e| e.ignorada_en_salud);
    let nuevo_estado = !todos_ignorados;
    let ahora = chrono::Utc::now().timestamp_millis();

    for e in entradas.iter_mut() {
        if ids.contains(&e.id) {
            e.ignorada_en_salud = nuevo_estado;
            e.modificada_en = ahora;
        }
    }
    nuevo_estado
}
