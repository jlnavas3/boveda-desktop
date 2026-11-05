/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Estructura raíz serializada en el archivo `.bvda` (ContenidoBoveda).
//! Compatible al 100% con ContenidoBoveda.kt de Bóveda Local Android.

use crate::models::categoria::Categoria;
use crate::models::entrada::Entrada;
use crate::models::perfil::Perfil;
use crate::models::vault_config::ConfiguracionBovedaExportable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContenidoBoveda {
    #[serde(default = "default_version")]
    pub version: i32,
    #[serde(default)]
    pub entradas: Vec<Entrada>,
    #[serde(default)]
    pub papelera: Vec<Entrada>,
    #[serde(default, rename = "colecciones", alias = "categorias")]
    pub categorias: Vec<Categoria>,
    #[serde(default, rename = "identidades", alias = "perfiles")]
    pub perfiles: Vec<Perfil>,
    #[serde(default)]
    pub configuracion: Option<ConfiguracionBovedaExportable>,
    #[serde(rename = "aliasDispositivo", default, skip_serializing_if = "Option::is_none")]
    pub alias_dispositivo: Option<String>,
}

fn default_version() -> i32 {
    1
}

impl Default for ContenidoBoveda {
    fn default() -> Self {
        Self {
            version: default_version(),
            entradas: Vec::new(),
            papelera: Vec::new(),
            categorias: Vec::new(),
            perfiles: Vec::new(),
            configuracion: Some(ConfiguracionBovedaExportable::default()),
            alias_dispositivo: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contenido_boveda_roundtrip() {
        let mut boveda = ContenidoBoveda::default();
        boveda.entradas.push(Entrada {
            id: "e-1".into(),
            titulo: "Google".into(),
            usuario: "user@example.com".into(),
            contrasena: "s3cr3t".into(),
            ..Default::default()
        });

        let json = serde_json::to_string(&boveda).unwrap();
        assert!(json.contains("\"colecciones\":[]"));
        assert!(json.contains("\"identidades\":[]"));
        assert!(json.contains("\"configuracion\":{"));

        let des: ContenidoBoveda = serde_json::from_str(&json).unwrap();
        assert_eq!(boveda, des);
    }
}
