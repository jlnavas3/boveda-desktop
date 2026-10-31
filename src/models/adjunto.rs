/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Archivos adjuntos cifrados dentro del contenedor de la entrada.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdjuntoBoveda {
    pub id: String,
    pub nombre: String,
    #[serde(default = "default_mime_type")]
    pub mime_type: String,
    #[serde(default)]
    pub tamano_bytes: i64,
    #[serde(default)]
    pub contenido_base64: String,
    #[serde(default)]
    pub fecha_subida: i64,
}

fn default_mime_type() -> String {
    "application/octet-stream".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adjunto_boveda_serializacion() {
        let adjunto = AdjuntoBoveda {
            id: "adj-1".into(),
            nombre: "clave.pub".into(),
            mime_type: "text/plain".into(),
            tamano_bytes: 128,
            contenido_base64: "c3NoLWVkMjU1MTkg...".into(),
            fecha_subida: 1700000000000,
        };
        let json = serde_json::to_string(&adjunto).unwrap();
        assert!(json.contains("tamanoBytes"));
        let des: AdjuntoBoveda = serde_json::from_str(&json).unwrap();
        assert_eq!(adjunto, des);
    }
}
