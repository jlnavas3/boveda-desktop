/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Rotación rápida de contraseña para una entrada desde Salud de la Bóveda,
//! preservando la contraseña anterior en `historial_contrasenas`.

use crate::models::{CambioContrasena, Entrada};

pub fn rotar_contrasena_rapida(entrada: &mut Entrada, nueva_contrasena: &str) -> bool {
    let nueva = nueva_contrasena.trim();
    if nueva.is_empty() || entrada.contrasena == nueva {
        return false;
    }

    let ahora = chrono::Utc::now().timestamp_millis();
    if !entrada.contrasena.is_empty() {
        entrada.historial_contrasenas.push(CambioContrasena {
            contrasena: entrada.contrasena.clone(),
            cambiada_en: ahora,
        });
    }

    entrada.contrasena = nueva.to_string();
    entrada.modificada_en = ahora;
    true
}
