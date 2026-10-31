/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Registro histórico de contraseñas anteriores asociadas a una entrada.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CambioContrasena {
    pub contrasena: String,
    pub cambiada_en: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cambio_contrasena_serializacion() {
        let cambio = CambioContrasena {
            contrasena: "claveAntigua123".into(),
            cambiada_en: 1700000000000,
        };
        let json = serde_json::to_string(&cambio).unwrap();
        assert!(json.contains("cambiadaEn"));
        let des: CambioContrasena = serde_json::from_str(&json).unwrap();
        assert_eq!(cambio, des);
    }
}
