/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Modelo de datos inmutable para llaves de acceso (Passkeys FIDO2 / WebAuthn).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatosPasskey {
    pub rp_id: String,
    pub rp_name: String,
    pub user_handle: String,
    pub cred_id: String,
    pub clave_privada: String,
    #[serde(default = "default_algoritmo_passkey")]
    pub algoritmo: String,
    #[serde(default)]
    pub usuario: String,
    #[serde(default)]
    pub creada_en: i64,
}

fn default_algoritmo_passkey() -> String {
    "ES256".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passkey_defaults() {
        let json = r#"{
            "rpId": "google.com",
            "rpName": "Google",
            "userHandle": "dXNlcg==",
            "credId": "Y3JlZA==",
            "clavePrivada": "cHJpdg=="
        }"#;
        let passkey: DatosPasskey = serde_json::from_str(json).unwrap();
        assert_eq!(passkey.rp_id, "google.com");
        assert_eq!(passkey.algoritmo, "ES256");
    }
}
