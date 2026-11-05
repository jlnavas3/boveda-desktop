/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Función desacoplada para restaurar una contraseña del historial como contraseña activa,
//! preservando la contraseña anterior dentro del mismo historial (paridad con Android).

use crate::models::cambio_contrasena::CambioContrasena;
use crate::models::Entrada;

/// Restaura `clave_restaurar` como la contraseña principal de `entrada` y conserva la actual en `historial_contrasenas`.
pub fn restaurar_contrasena_historial(entrada: &mut Entrada, clave_restaurar: &str) -> bool {
    if clave_restaurar.is_empty() || entrada.contrasena == clave_restaurar {
        return false;
    }

    let ahora = chrono::Utc::now().timestamp_millis();
    let clave_anterior = entrada.contrasena.clone();

    entrada
        .historial_contrasenas
        .retain(|c| c.contrasena != clave_restaurar);

    if !clave_anterior.is_empty()
        && !entrada
            .historial_contrasenas
            .iter()
            .any(|c| c.contrasena == clave_anterior)
    {
        entrada.historial_contrasenas.insert(
            0,
            CambioContrasena {
                contrasena: clave_anterior,
                cambiada_en: ahora,
            },
        );
    }

    entrada.contrasena = clave_restaurar.to_string();
    entrada.modificada_en = ahora;
    true
}
