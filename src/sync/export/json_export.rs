/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Exportador a formato JSON plano con estructura legible.

use crate::crypto::BvdaError;
use crate::models::ContenidoBoveda;

/// Exporta el contenido íntegro de la bóveda a un String en formato JSON formateado.
pub fn exportar_json_plano(boveda: &ContenidoBoveda) -> Result<String, BvdaError> {
    serde_json::to_string_pretty(boveda).map_err(BvdaError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exportar_json_plano() {
        let boveda = ContenidoBoveda::default();
        let json = exportar_json_plano(&boveda).unwrap();
        assert!(json.contains("\"version\": 1"));
        assert!(json.contains("\"entradas\": []"));
    }
}
