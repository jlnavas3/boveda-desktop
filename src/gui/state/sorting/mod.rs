/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Módulo de ordenación de entradas del listado principal.

pub mod sort_criterion;
pub mod sort_entries;

pub use sort_criterion::CriterioOrdenacion;
pub use sort_entries::ordenar_entradas_referencias;
