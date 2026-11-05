/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Operaciones de papelera y borrado definitivo en lote sobre las entradas seleccionadas.

use std::collections::HashSet;

use crate::models::Entrada;

/// Ejecuta `"MOVER_PAPELERA"`, `"RESTAURAR"` o `"ELIMINAR_DEFINITIVO"` sobre `ids`.
pub fn procesar_papelera_lote(
    entradas: &mut Vec<Entrada>,
    ids: &HashSet<String>,
    accion: &str,
) -> usize {
    let ahora = chrono::Utc::now().timestamp_millis();
    match accion {
        "MOVER_PAPELERA" => {
            let mut count = 0usize;
            for e in entradas.iter_mut() {
                if ids.contains(&e.id) && e.eliminada_en == 0 {
                    e.eliminada_en = ahora;
                    e.modificada_en = ahora;
                    e.revision += 1;
                    count += 1;
                }
            }
            count
        }
        "RESTAURAR" => {
            let mut count = 0usize;
            for e in entradas.iter_mut() {
                if ids.contains(&e.id) && e.eliminada_en > 0 {
                    e.eliminada_en = 0;
                    e.modificada_en = ahora;
                    e.revision += 1;
                    count += 1;
                }
            }
            count
        }
        "ELIMINAR_DEFINITIVO" => {
            let antes = entradas.len();
            entradas.retain(|e| !ids.contains(&e.id));
            antes.saturating_sub(entradas.len())
        }
        _ => 0,
    }
}
