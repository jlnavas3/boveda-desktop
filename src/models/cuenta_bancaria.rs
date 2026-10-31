/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Modelo de datos inmutable para cuentas bancarias e IBAN.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatosCuentaBancaria {
    #[serde(default)]
    pub banco: String,
    #[serde(default)]
    pub titular: String,
    #[serde(default = "default_tipo_cuenta")]
    pub tipo_cuenta: String,
    #[serde(default, alias = "iban")]
    pub numero_cuenta: String,
    #[serde(default)]
    pub swift_bic: String,
    #[serde(default)]
    pub cbu_clabe: String,
}

fn default_tipo_cuenta() -> String {
    "Corriente".into()
}

impl Default for DatosCuentaBancaria {
    fn default() -> Self {
        Self {
            banco: String::new(),
            titular: String::new(),
            tipo_cuenta: default_tipo_cuenta(),
            numero_cuenta: String::new(),
            swift_bic: String::new(),
            cbu_clabe: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cuenta_bancaria_alias_iban() {
        let json = r#"{
            "banco": "Banco Santander",
            "titular": "Juan Perez",
            "iban": "ES9121000418450200051332",
            "swiftBic": "BSCHESMMXXX"
        }"#;
        let cuenta: DatosCuentaBancaria = serde_json::from_str(json).unwrap();
        assert_eq!(cuenta.banco, "Banco Santander");
        assert_eq!(cuenta.numero_cuenta, "ES9121000418450200051332");
        assert_eq!(cuenta.tipo_cuenta, "Corriente");
    }
}
