/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Estado desacoplado de gestión de zoom y escalado automático por resolución.

use crate::config::defaults::escalado_ventana::{
    ESCALA_AUTO_FHD_1080P, FACTOR_FINAL_MAX, FACTOR_FINAL_MIN, MULTIPLICADOR_USUARIO_DEFECTO,
    MULTIPLICADOR_USUARIO_MAX, MULTIPLICADOR_USUARIO_MIN, PASO_ZOOM_USUARIO,
};
use crate::gui::zoom::calcular_escala_automatica;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZoomState {
    pub factor_auto_resolucion: f32,
    pub multiplicador_usuario: f32,
    pub ultimo_ancho_fisico: u32,
    pub ultimo_alto_fisico: u32,
}

impl Default for ZoomState {
    fn default() -> Self {
        Self {
            factor_auto_resolucion: ESCALA_AUTO_FHD_1080P,
            multiplicador_usuario: MULTIPLICADOR_USUARIO_DEFECTO,
            ultimo_ancho_fisico: 0,
            ultimo_alto_fisico: 0,
        }
    }
}

impl ZoomState {
    /// Actualiza la resolución física detectada y recalcula el factor base automático.
    /// Retorna `true` si el factor automático de resolución cambió.
    pub fn actualizar_resolucion(&mut self, ancho_fisico: u32, alto_fisico: u32) -> bool {
        self.ultimo_ancho_fisico = ancho_fisico;
        self.ultimo_alto_fisico = alto_fisico;
        let nuevo_auto = calcular_escala_automatica(ancho_fisico, alto_fisico);
        let cambio = (nuevo_auto - self.factor_auto_resolucion).abs() > 0.01;
        self.factor_auto_resolucion = nuevo_auto;
        cambio
    }

    /// Incrementa el zoom manual del usuario en un paso (`Ctrl +`).
    pub fn aumentar_zoom(&mut self) -> f32 {
        let nuevo = (self.multiplicador_usuario + PASO_ZOOM_USUARIO)
            .clamp(MULTIPLICADOR_USUARIO_MIN, MULTIPLICADOR_USUARIO_MAX);
        self.multiplicador_usuario = (nuevo * 100.0).round() / 100.0;
        self.factor_final()
    }

    /// Reduce el zoom manual del usuario en un paso (`Ctrl -`).
    pub fn reducir_zoom(&mut self) -> f32 {
        let nuevo = (self.multiplicador_usuario - PASO_ZOOM_USUARIO)
            .clamp(MULTIPLICADOR_USUARIO_MIN, MULTIPLICADOR_USUARIO_MAX);
        self.multiplicador_usuario = (nuevo * 100.0).round() / 100.0;
        self.factor_final()
    }

    /// Restaura el zoom manual del usuario al 100% de la escala automática de su resolución (`Ctrl 0`).
    pub fn restaurar_zoom(&mut self) -> f32 {
        self.multiplicador_usuario = MULTIPLICADOR_USUARIO_DEFECTO;
        self.factor_final()
    }

    /// Calcula el factor de escala combinado final (auto_resolucion * multiplicador_usuario).
    pub fn factor_final(&self) -> f32 {
        (self.factor_auto_resolucion * self.multiplicador_usuario)
            .clamp(FACTOR_FINAL_MIN, FACTOR_FINAL_MAX)
    }

    /// Porcentaje legible para mostrar al usuario en notificaciones (ej. 100%, 110%, 90%).
    pub fn porcentaje_usuario(&self) -> i32 {
        (self.multiplicador_usuario * 100.0).round() as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ciclo_zoom_y_resolucion() {
        let mut st = ZoomState::default();
        assert!(st.actualizar_resolucion(1280, 720));
        assert!((st.factor_final() - 0.75).abs() < 0.001);
        assert_eq!(st.porcentaje_usuario(), 100);

        let f_mas = st.aumentar_zoom();
        assert_eq!(st.porcentaje_usuario(), 110);
        assert!((f_mas - (0.75 * 1.10)).abs() < 0.001);

        let f_reset = st.restaurar_zoom();
        assert_eq!(st.porcentaje_usuario(), 100);
        assert!((f_reset - 0.75).abs() < 0.001);
    }
}
