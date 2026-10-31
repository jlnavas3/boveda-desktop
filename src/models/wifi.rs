/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Modelo de datos inmutable para redes Wi-Fi protegidas.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatosRedWifi {
    #[serde(default)]
    pub ssid: String,
    #[serde(default)]
    pub contrasena: String,
    #[serde(default = "default_tipo_seguridad", alias = "seguridad")]
    pub tipo_seguridad: String,
    #[serde(default)]
    pub oculta: bool,
}

fn default_tipo_seguridad() -> String {
    "WPA".into()
}

impl Default for DatosRedWifi {
    fn default() -> Self {
        Self {
            ssid: String::new(),
            contrasena: String::new(),
            tipo_seguridad: default_tipo_seguridad(),
            oculta: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wifi_alias_seguridad() {
        let json = r#"{
            "ssid": "MiFibra_5G",
            "seguridad": "WPA3",
            "oculta": false
        }"#;
        let wifi: DatosRedWifi = serde_json::from_str(json).unwrap();
        assert_eq!(wifi.ssid, "MiFibra_5G");
        assert_eq!(wifi.tipo_seguridad, "WPA3");
        assert!(!wifi.oculta);
    }
}
