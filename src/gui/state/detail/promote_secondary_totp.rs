/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Función desacoplada para establecer un código 2FA (TOTP) secundario como principal,
//! conservando el TOTP principal anterior dentro de `totps_secundarios`.

use crate::models::totp_secundario::TotpSecundario;
use crate::models::Entrada;

/// Intercambia el TOTP secundario identificado por `totp_id` con el TOTP principal de `entrada`.
pub fn promover_totp_secundario(entrada: &mut Entrada, totp_id: &str) -> bool {
    let pos = match entrada
        .totps_secundarios
        .iter()
        .position(|t| t.id == totp_id)
    {
        Some(idx) => idx,
        None => return false,
    };

    let nuevo_principal = entrada.totps_secundarios.remove(pos);

    if let Some(ref sec_actual) = entrada.secreto_totp {
        if !sec_actual.trim().is_empty() {
            let emisor_ant = if entrada.totp_emisor.trim().is_empty() {
                entrada.titulo.clone()
            } else {
                entrada.totp_emisor.clone()
            };
            entrada.totps_secundarios.insert(
                0,
                TotpSecundario {
                    id: uuid::Uuid::new_v4().to_string(),
                    secreto: sec_actual.clone(),
                    emisor: emisor_ant,
                    periodo: entrada.totp_periodo,
                    digitos: entrada.totp_digitos,
                    algoritmo: entrada.totp_algoritmo.clone(),
                    agregado_en: chrono::Utc::now().timestamp_millis(),
                },
            );
        }
    }

    entrada.secreto_totp = Some(nuevo_principal.secreto);
    entrada.totp_emisor = nuevo_principal.emisor;
    entrada.totp_periodo = nuevo_principal.periodo;
    entrada.totp_digitos = nuevo_principal.digitos;
    entrada.totp_algoritmo = nuevo_principal.algoritmo;
    entrada.modificada_en = chrono::Utc::now().timestamp_millis();
    true
}
