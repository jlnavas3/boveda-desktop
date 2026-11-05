/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Criterio de ordenación tipado para el listado principal de entradas.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CriterioOrdenacion {
    NombreAz,
    NombreZa,
    FechaModificacion,
    FechaCreacion,
    UltimoUso,
    FrecuenciaUso,
}

impl CriterioOrdenacion {
    pub fn desde_str(valor: &str) -> Self {
        match valor.trim().to_uppercase().as_str() {
            "NOMBRE_ZA" | "TITULO_DESC" => Self::NombreZa,
            "FECHA_MODIFICACION" | "RECIENTES" | "MODIFICACION" => Self::FechaModificacion,
            "FECHA_CREACION" | "ANTIGUOS" | "CREACION" => Self::FechaCreacion,
            "ULTIMO_USO" => Self::UltimoUso,
            "FRECUENCIA_USO" | "MAS_USADOS" => Self::FrecuenciaUso,
            _ => Self::NombreAz,
        }
    }
}
