/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Submódulo de importación de credenciales desde fuentes externas.

pub mod bitwarden;
pub mod csv_import;

pub use bitwarden::importar_bitwarden_json;
pub use csv_import::{detectar_delimitador, importar_csv, tokenizar_csv};
