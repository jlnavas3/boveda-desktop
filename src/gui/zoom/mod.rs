/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Submódulo desacoplado de auto-escalado por resolución y control de zoom global.

pub mod apply_zoom;
pub mod auto_scale;
pub mod zoom_state;

pub use apply_zoom::aplicar_zoom_ventana;
pub use auto_scale::calcular_escala_automatica;
pub use zoom_state::ZoomState;
