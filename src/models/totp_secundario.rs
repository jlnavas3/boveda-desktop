/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Códigos TOTP (2FA) secundarios para cuentas con múltiples factores.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TotpSecundario {
    pub id: String,
    pub secreto: String,
    #[serde(default)]
    pub emisor: String,
    #[serde(default = "default_totp_digitos")]
    pub digitos: i32,
    #[serde(default = "default_totp_periodo")]
    pub periodo: i32,
    #[serde(default = "default_totp_algoritmo")]
    pub algoritmo: String,
    #[serde(default)]
    pub agregado_en: i64,
}

fn default_totp_digitos() -> i32 {
    6
}

fn default_totp_periodo() -> i32 {
    30
}

fn default_totp_algoritmo() -> String {
    "HmacSHA1".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_totp_secundario_defaults() {
        let json = r#"{"id":"1","secreto":"JBSWY3DPEHPK3PXP"}"#;
        let totp: TotpSecundario = serde_json::from_str(json).unwrap();
        assert_eq!(totp.digitos, 6);
        assert_eq!(totp.periodo, 30);
        assert_eq!(totp.algoritmo, "HmacSHA1");
    }
}
