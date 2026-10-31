/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Modelo de datos inmutable para documentos de identidad personal.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatosDocumentoIdentidad {
    #[serde(default = "default_tipo_documento")]
    pub tipo_documento: String,
    #[serde(default)]
    pub numero: String,
    #[serde(default)]
    pub nombre_completo: String,
    #[serde(default, alias = "fechaEmision")]
    pub expedicion: String,
    #[serde(default, alias = "fechaExpiracion")]
    pub caducidad: String,
    #[serde(default, alias = "pais")]
    pub pais_emisor: String,
}

fn default_tipo_documento() -> String {
    "CI".into()
}

impl Default for DatosDocumentoIdentidad {
    fn default() -> Self {
        Self {
            tipo_documento: default_tipo_documento(),
            numero: String::new(),
            nombre_completo: String::new(),
            expedicion: String::new(),
            caducidad: String::new(),
            pais_emisor: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identidad_doc_deserializacion_alias() {
        let json = r#"{
            "tipoDocumento": "DNI",
            "numero": "12345678Z",
            "pais": "España",
            "fechaEmision": "2020-01-15",
            "fechaExpiracion": "2030-01-15"
        }"#;
        let doc: DatosDocumentoIdentidad = serde_json::from_str(json).unwrap();
        assert_eq!(doc.tipo_documento, "DNI");
        assert_eq!(doc.numero, "12345678Z");
        assert_eq!(doc.pais_emisor, "España");
        assert_eq!(doc.expedicion, "2020-01-15");
        assert_eq!(doc.caducidad, "2030-01-15");
    }
}
