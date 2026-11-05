/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Unificación sin pérdida de un grupo de duplicados a partir de sus IDs separados por coma.
//! Reutiliza `unificar_entradas_seleccionadas` para garantizar cero duplicidad de código.

use std::collections::HashSet;

use crate::gui::state::selection::unificar_entradas_seleccionadas;
use crate::models::ContenidoBoveda;

pub fn unificar_grupo_duplicado(
    boveda: &mut ContenidoBoveda,
    ids_csv: &str,
) -> Option<String> {
    let ids: HashSet<String> = ids_csv
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if ids.len() < 2 {
        return None;
    }

    unificar_entradas_seleccionadas(boveda, &ids, None)
}
