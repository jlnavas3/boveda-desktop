/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Función desacoplada para aplicar el factor de escala a toda la ventana de Slint.

use slint::platform::WindowEvent;
use slint::Window;

/// Aplica el factor de escala global a la ventana de Slint, escalando de forma uniforme
/// absolutamente todos los componentes, columnas, botones, badges, íconos y textos,
/// preservando intactas las dimensiones físicas exteriores de la ventana.
pub fn aplicar_zoom_ventana(window: &Window, factor_escala: f32) {
    let tamano_fisico = window.size();
    window.dispatch_event(WindowEvent::ScaleFactorChanged {
        scale_factor: factor_escala,
    });
    if tamano_fisico.width > 0 && tamano_fisico.height > 0 {
        let tamano_logico = tamano_fisico.to_logical(factor_escala);
        window.dispatch_event(WindowEvent::Resized {
            size: tamano_logico,
        });
        window.set_size(tamano_fisico);
    }
}
