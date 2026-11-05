/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Limpieza y unificación masiva de todos los grupos de duplicados activos bajo el filtro actual.

use crate::gui::state::duplicates::detect_duplicate_groups::detectar_grupos_duplicados;
use crate::gui::state::duplicates::merge_duplicate_group::unificar_grupo_duplicado;
use crate::models::ContenidoBoveda;

pub fn unificar_todos_los_duplicados(
    boveda: &mut ContenidoBoveda,
    filtro_str: &str,
) -> usize {
    let grupos = detectar_grupos_duplicados(boveda, filtro_str);
    let mut grupos_unificados = 0usize;

    for grupo in grupos {
        if unificar_grupo_duplicado(boveda, grupo.ids_csv.as_str()).is_some() {
            grupos_unificados += 1;
        }
    }

    grupos_unificados
}
