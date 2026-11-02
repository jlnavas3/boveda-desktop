/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Función desacoplada para aplicar el factor de escala a toda la ventana de Slint.

use slint::platform::WindowEvent;
use slint::Window;

/// Aplica el factor de escala global a la ventana de Slint, escalando de forma uniforme
/// absolutamente todos los componentes, columnas, botones, badges, íconos y textos.
pub fn aplicar_zoom_ventana(window: &Window, factor_escala: f32) {
    window.dispatch_event(WindowEvent::ScaleFactorChanged {
        scale_factor: factor_escala,
    });
}
