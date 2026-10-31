/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Modelo de datos inmutable para contactos seguros y domicilios.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatosContacto {
    #[serde(default)]
    pub nombre: String,
    #[serde(default)]
    pub telefono_secundario: String,
    #[serde(default)]
    pub correo: String,
    #[serde(default)]
    pub direccion: String,
    #[serde(default)]
    pub codigo_postal: String,
    #[serde(default)]
    pub ciudad: String,
    #[serde(default)]
    pub provincia: String,
    #[serde(default)]
    pub pais: String,
    #[serde(default)]
    pub empresa: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contacto_roundtrip() {
        let contacto = DatosContacto {
            nombre: "Maria Lopez".into(),
            telefono_secundario: "+34612345678".into(),
            correo: "maria@ejemplo.com".into(),
            direccion: "Calle Mayor 1".into(),
            codigo_postal: "28001".into(),
            ciudad: "Madrid".into(),
            provincia: "Madrid".into(),
            pais: "Espana".into(),
            empresa: "Acme Corp".into(),
        };
        let json = serde_json::to_string(&contacto).unwrap();
        let des: DatosContacto = serde_json::from_str(&json).unwrap();
        assert_eq!(contacto, des);
    }
}
