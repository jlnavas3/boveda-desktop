/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Módulo de detección y unificación sin pérdida de entradas duplicadas.

pub mod count_secondary_duplicates;
pub mod detect_duplicate_groups;
pub mod duplicate_filter_kind;
pub mod duplicate_normalizer;
pub mod merge_all_duplicate_groups;
pub mod merge_duplicate_group;
pub mod native_duplicate_key;

pub use count_secondary_duplicates::contar_entradas_secundarias_duplicadas;
pub use detect_duplicate_groups::detectar_grupos_duplicados;
pub use duplicate_filter_kind::FiltroDuplicados;
pub use merge_all_duplicate_groups::unificar_todos_los_duplicados;
pub use merge_duplicate_group::unificar_grupo_duplicado;
