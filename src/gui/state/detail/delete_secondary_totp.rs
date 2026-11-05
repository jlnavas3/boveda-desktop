/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Función desacoplada para eliminar un código 2FA (TOTP) secundario de una entrada.

use crate::models::Entrada;

/// Elimina permanentemente el TOTP secundario con `totp_id` de `entrada`.
pub fn eliminar_totp_secundario(entrada: &mut Entrada, totp_id: &str) -> bool {
    let len_inicial = entrada.totps_secundarios.len();
    entrada.totps_secundarios.retain(|t| t.id != totp_id);
    if entrada.totps_secundarios.len() < len_inicial {
        entrada.modificada_en = chrono::Utc::now().timestamp_millis();
        true
    } else {
        false
    }
}
