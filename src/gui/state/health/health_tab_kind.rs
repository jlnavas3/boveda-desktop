/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Pestañas de auditoría de Salud de la Bóveda (idéntico a PantallaSaludBoveda en Android).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PestanaSalud {
    Caducadas,
    Repetidas,
    Comunes,
    Debiles,
    Antiguas,
    Ignoradas,
}

impl PestanaSalud {
    pub fn desde_str(valor: &str) -> Self {
        match valor.trim().to_uppercase().as_str() {
            "CADUCADAS" => Self::Caducadas,
            "REPETIDAS" => Self::Repetidas,
            "COMUNES" => Self::Comunes,
            "DEBILES" => Self::Debiles,
            "ANTIGUAS" => Self::Antiguas,
            "IGNORADAS" => Self::Ignoradas,
            _ => Self::Repetidas,
        }
    }
}
