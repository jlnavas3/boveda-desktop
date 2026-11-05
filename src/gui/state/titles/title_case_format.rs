/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Formatos de capitalización para el Normalizador Interactivo de Títulos.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatoTitulo {
    Capitalizado,
    Mayusculas,
    Minusculas,
    Dominio,
}

impl FormatoTitulo {
    pub fn desde_str(valor: &str) -> Self {
        match valor.trim().to_uppercase().as_str() {
            "MAYUSCULAS" => Self::Mayusculas,
            "MINUSCULAS" => Self::Minusculas,
            "DOMINIO" => Self::Dominio,
            _ => Self::Capitalizado,
        }
    }

    pub fn aplicar(&self, nombre_limpio: &str, primera_url: &str) -> String {
        match self {
            Self::Capitalizado => nombre_limpio.to_string(),
            Self::Mayusculas => nombre_limpio.to_uppercase(),
            Self::Minusculas => nombre_limpio.to_lowercase(),
            Self::Dominio => {
                let sin_esquema = primera_url
                    .trim()
                    .trim_start_matches("https://")
                    .trim_start_matches("http://")
                    .trim_start_matches("www.");
                let host = sin_esquema.split('/').next().unwrap_or("").trim();
                if !host.is_empty() && !host.starts_with("android://") {
                    host.to_lowercase()
                } else {
                    nombre_limpio.to_lowercase()
                }
            }
        }
    }
}
