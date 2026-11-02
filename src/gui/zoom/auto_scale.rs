/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Función pura desacoplada para calcular el factor de escala base automático
//! según las dimensiones físicas de la ventana o monitor (1280x720, 1920x1080, 2560x1440, 3840x2160).

use crate::config::defaults::escalado_ventana::{
    ESCALA_AUTO_FHD_1080P, ESCALA_AUTO_HD_720P, ESCALA_AUTO_QHD_1440P, ESCALA_AUTO_UHD_2160P,
    UMBRAL_ANCHO_HD, UMBRAL_ANCHO_QHD, UMBRAL_ANCHO_UHD_4K,
};

/// Calcula el factor de escala base automático en función del ancho físico en píxeles.
pub fn calcular_escala_automatica(ancho_fisico_px: u32, _alto_fisico_px: u32) -> f32 {
    if ancho_fisico_px == 0 {
        return ESCALA_AUTO_FHD_1080P;
    }

    if ancho_fisico_px <= UMBRAL_ANCHO_HD {
        ESCALA_AUTO_HD_720P
    } else if ancho_fisico_px >= UMBRAL_ANCHO_UHD_4K {
        ESCALA_AUTO_UHD_2160P
    } else if ancho_fisico_px >= UMBRAL_ANCHO_QHD {
        ESCALA_AUTO_QHD_1440P
    } else {
        ESCALA_AUTO_FHD_1080P
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resoluciones_estandar() {
        assert!((calcular_escala_automatica(1280, 720) - 0.75).abs() < f32::EPSILON);
        assert!((calcular_escala_automatica(1366, 768) - 0.75).abs() < f32::EPSILON);
        assert!((calcular_escala_automatica(1920, 1080) - 1.00).abs() < f32::EPSILON);
        assert!((calcular_escala_automatica(2560, 1440) - 1.25).abs() < f32::EPSILON);
        assert!((calcular_escala_automatica(3840, 2160) - 1.75).abs() < f32::EPSILON);
    }
}
