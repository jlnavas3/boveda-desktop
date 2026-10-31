/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Categoría o carpeta temática definida por el usuario.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Categoria {
    pub id: String,
    pub nombre: String,
    #[serde(default = "default_icono_categoria")]
    pub icono: String,
    #[serde(default)]
    pub color_hex: Option<String>,
    #[serde(default)]
    pub color: Option<i64>,
    #[serde(default)]
    pub creada_en: i64,
    #[serde(default)]
    pub modificada_en: i64,
}

fn default_icono_categoria() -> String {
    "carpeta".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categoria_roundtrip() {
        let cat = Categoria {
            id: "cat-1".into(),
            nombre: "Finanzas".into(),
            icono: "bank".into(),
            color_hex: Some("#2563EB".into()),
            color: None,
            creada_en: 1700000000000,
            modificada_en: 1700000000000,
        };
        let json = serde_json::to_string(&cat).unwrap();
        assert!(json.contains("colorHex"));
        let des: Categoria = serde_json::from_str(&json).unwrap();
        assert_eq!(cat, des);
    }
}
