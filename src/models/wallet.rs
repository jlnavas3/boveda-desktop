/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Modelo de datos inmutable para billeteras de criptomonedas y frases semilla.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatosCriptoWallet {
    #[serde(default = "default_red_cripto")]
    pub red: String,
    #[serde(default, alias = "clavePublica")]
    pub direccion_publica: String,
    #[serde(default)]
    pub frase_semilla: String,
    #[serde(default)]
    pub clave_privada: String,
}

fn default_red_cripto() -> String {
    "Bitcoin".into()
}

impl Default for DatosCriptoWallet {
    fn default() -> Self {
        Self {
            red: default_red_cripto(),
            direccion_publica: String::new(),
            frase_semilla: String::new(),
            clave_privada: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_alias_clave_publica() {
        let json = r#"{
            "fraseSemilla": "word1 word2 word3",
            "clavePublica": "0x71C456",
            "red": "Ethereum"
        }"#;
        let wallet: DatosCriptoWallet = serde_json::from_str(json).unwrap();
        assert_eq!(wallet.red, "Ethereum");
        assert_eq!(wallet.direccion_publica, "0x71C456");
        assert_eq!(wallet.frase_semilla, "word1 word2 word3");
    }
}
