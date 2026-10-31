/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Perfil organizativo de usuario (identidad).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Perfil {
    pub id: String,
    pub nombre: String,
    #[serde(default)]
    pub correo_principal: String,
    #[serde(default)]
    pub correos_secundarios: Vec<String>,
    #[serde(default)]
    pub tipo_documento: Option<String>,
    #[serde(default)]
    pub numero_documento: Option<String>,
    #[serde(default = "default_preferencia_identificador")]
    pub preferencia_identificador: String,
    #[serde(default)]
    pub color_hex: Option<String>,
    #[serde(default)]
    pub color: Option<i64>,
    #[serde(default = "default_icono_perfil")]
    pub icono: String,
    #[serde(default)]
    pub tarjetas_ids: Vec<String>,
    #[serde(default)]
    pub tarjeta_predeterminada_id: Option<String>,
    #[serde(default)]
    pub cuentas_ids: Vec<String>,
    #[serde(default)]
    pub cuenta_predeterminada_id: Option<String>,
    #[serde(default)]
    pub contactos_ids: Vec<String>,
    #[serde(default)]
    pub contacto_predeterminado_id: Option<String>,
    #[serde(default)]
    pub documentos_ids: Vec<String>,
    #[serde(default)]
    pub documento_predeterminado_id: Option<String>,
    #[serde(default)]
    pub creada_en: i64,
    #[serde(default)]
    pub modificada_en: i64,
}

fn default_preferencia_identificador() -> String {
    "correo".into()
}

fn default_icono_perfil() -> String {
    "person".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perfil_defaults() {
        let json = r#"{"id":"perfil-1","nombre":"Personal"}"#;
        let p: Perfil = serde_json::from_str(json).unwrap();
        assert_eq!(p.preferencia_identificador, "correo");
        assert_eq!(p.icono, "person");
    }
}
