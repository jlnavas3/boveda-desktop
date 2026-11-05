/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Función desacoplada para eliminar una contraseña anterior del historial de una entrada.

use crate::models::Entrada;

/// Elimina permanentemente `clave_eliminar` del historial de contraseñas de `entrada`.
pub fn eliminar_contrasena_historial(entrada: &mut Entrada, clave_eliminar: &str) -> bool {
    let len_inicial = entrada.historial_contrasenas.len();
    entrada
        .historial_contrasenas
        .retain(|c| c.contrasena != clave_eliminar);
    if entrada.historial_contrasenas.len() < len_inicial {
        entrada.modificada_en = chrono::Utc::now().timestamp_millis();
        true
    } else {
        false
    }
}
