/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Módulo del Normalizador Interactivo de Títulos.

pub mod apply_title_normalization;
pub mod preview_title_normalization;
pub mod title_case_format;
pub mod title_compromised_check;

pub use apply_title_normalization::aplicar_normalizacion_titulos;
pub use preview_title_normalization::previsualizar_normalizacion_titulos;
pub use title_case_format::FormatoTitulo;
pub use title_compromised_check::{es_entrada_protegida_manual, es_titulo_comprometido};
