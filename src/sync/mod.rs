/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Módulo de sincronización, fusión inteligente e interoperabilidad externa.

pub mod export;
pub mod import;
pub mod merge;

pub use export::{exportar_csv, exportar_json_plano};
pub use import::{detectar_delimitador, importar_bitwarden_json, importar_csv};
pub use merge::{
    consolidar_passkey, fusionar_bovedas, fusionar_entradas, fusionar_submodelos,
    son_misma_passkey, ResultadoFusionNativa,
};
