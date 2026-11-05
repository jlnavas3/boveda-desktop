/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Función desacoplada para establecer una llave de acceso secundaria como principal,
//! moviendo la llave principal anterior a `passkeys_secundarias` sin pérdida de datos.

use crate::models::Entrada;

/// Intercambia la passkey secundaria identificada por `cred_id` con la passkey principal de `entrada`.
pub fn promover_passkey_secundaria(entrada: &mut Entrada, cred_id: &str) -> bool {
    let pos = match entrada
        .passkeys_secundarias
        .iter()
        .position(|p| p.cred_id == cred_id)
    {
        Some(idx) => idx,
        None => return false,
    };

    let nueva_principal = entrada.passkeys_secundarias.remove(pos);
    if let Some(principal_anterior) = entrada.passkey.take() {
        if principal_anterior.cred_id != nueva_principal.cred_id {
            entrada.passkeys_secundarias.insert(0, principal_anterior);
        }
    }

    entrada.passkey = Some(nueva_principal);
    entrada.modificada_en = chrono::Utc::now().timestamp_millis();
    true
}
