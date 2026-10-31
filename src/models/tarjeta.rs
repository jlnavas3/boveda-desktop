/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Modelo de datos inmutable para tarjetas de crédito o débito.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatosTarjetaBancaria {
    #[serde(default)]
    pub titular: String,
    #[serde(default, alias = "numeroTarjeta")]
    pub numero: String,
    #[serde(default, alias = "fechaCaducidad")]
    pub vencimiento: String,
    #[serde(default)]
    pub cvv: String,
    #[serde(default)]
    pub pin: String,
    #[serde(default = "default_tipo_tarjeta", alias = "tipoTarjeta")]
    pub tipo: String,
    #[serde(default)]
    pub marca: String,
}

fn default_tipo_tarjeta() -> String {
    "Crédito".into()
}

impl Default for DatosTarjetaBancaria {
    fn default() -> Self {
        Self {
            titular: String::new(),
            numero: String::new(),
            vencimiento: String::new(),
            cvv: String::new(),
            pin: String::new(),
            tipo: default_tipo_tarjeta(),
            marca: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tarjeta_deserializacion_con_alias() {
        let json = r#"{
            "titular": "Juan Perez",
            "numeroTarjeta": "4532123456789012",
            "fechaCaducidad": "12/28",
            "cvv": "123"
        }"#;
        let tarjeta: DatosTarjetaBancaria = serde_json::from_str(json).unwrap();
        assert_eq!(tarjeta.titular, "Juan Perez");
        assert_eq!(tarjeta.numero, "4532123456789012");
        assert_eq!(tarjeta.vencimiento, "12/28");
        assert_eq!(tarjeta.tipo, "Crédito");
    }
}
