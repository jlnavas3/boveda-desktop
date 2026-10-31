/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Campo personalizado extensible para credenciales.

use crate::models::tipo_campo::TipoCampo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampoPersonalizado {
    pub id: String,
    #[serde(default)]
    pub etiqueta: String,
    #[serde(default)]
    pub valor: String,
    #[serde(default)]
    pub tipo: TipoCampo,
    #[serde(default)]
    pub es_sensible: bool,
    #[serde(default)]
    pub es_obligatorio: bool,
    #[serde(default)]
    pub formato: Option<String>,
}

impl CampoPersonalizado {
    pub fn es_sensible_efectivo(&self) -> bool {
        self.es_sensible || self.tipo == TipoCampo::Pin || self.tipo == TipoCampo::Oculto
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_campo_personalizado_roundtrip() {
        let campo = CampoPersonalizado {
            id: "cp-1".into(),
            etiqueta: "Pregunta Secreta".into(),
            valor: "Respuesta".into(),
            tipo: TipoCampo::Texto,
            es_sensible: false,
            es_obligatorio: false,
            formato: None,
        };
        let json = serde_json::to_string(&campo).unwrap();
        assert!(json.contains("esSensible"));
        let des: CampoPersonalizado = serde_json::from_str(&json).unwrap();
        assert_eq!(campo, des);
    }
}
