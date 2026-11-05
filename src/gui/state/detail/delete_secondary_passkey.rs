/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Función desacoplada para eliminar una llave de acceso secundaria de una entrada.

use crate::models::Entrada;

/// Elimina permanentemente la passkey secundaria con `cred_id` de `entrada`.
pub fn eliminar_passkey_secundaria(entrada: &mut Entrada, cred_id: &str) -> bool {
    let len_inicial = entrada.passkeys_secundarias.len();
    entrada
        .passkeys_secundarias
        .retain(|p| p.cred_id != cred_id);
    if entrada.passkeys_secundarias.len() < len_inicial {
        entrada.modificada_en = chrono::Utc::now().timestamp_millis();
        true
    } else {
        false
    }
}
