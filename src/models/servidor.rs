/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Modelo de datos inmutable para servidores remotos y claves SSH.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatosServidorSsh {
    #[serde(default)]
    pub host: String,
    #[serde(default = "default_puerto_ssh")]
    pub puerto: String,
    #[serde(default)]
    pub usuario: String,
    #[serde(default)]
    pub clave_privada: String,
    #[serde(default)]
    pub clave_publica: String,
}

fn default_puerto_ssh() -> String {
    "22".into()
}

impl Default for DatosServidorSsh {
    fn default() -> Self {
        Self {
            host: String::new(),
            puerto: default_puerto_ssh(),
            usuario: String::new(),
            clave_privada: String::new(),
            clave_publica: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_servidor_ssh_roundtrip() {
        let servidor = DatosServidorSsh {
            host: "192.168.1.100".into(),
            puerto: "2222".into(),
            usuario: "root".into(),
            clave_privada: "-----BEGIN OPENSSH PRIVATE KEY-----".into(),
            clave_publica: "ssh-ed25519 AAA...".into(),
        };
        let json = serde_json::to_string(&servidor).unwrap();
        let des: DatosServidorSsh = serde_json::from_str(&json).unwrap();
        assert_eq!(servidor, des);
    }
}
