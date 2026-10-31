/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Submódulo de exportación de credenciales a formatos abiertos.

pub mod csv_export;
pub mod json_export;

pub use csv_export::exportar_csv;
pub use json_export::exportar_json_plano;
