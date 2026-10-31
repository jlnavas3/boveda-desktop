/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Tipos de datos para campos personalizados adicionales.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TipoCampo {
    #[default]
    Texto,
    Numero,
    Decimal,
    Pin,
    Email,
    Url,
    Telefono,
    Fecha,
    Hora,
    Lista,
    Notas,
    Booleano,
    Oculto,
}

impl TipoCampo {
    pub const fn etiqueta(&self) -> &'static str {
        match self {
            Self::Texto => "Texto",
            Self::Numero => "Número",
            Self::Decimal => "Decimal",
            Self::Pin => "PIN",
            Self::Email => "Email",
            Self::Url => "URL",
            Self::Telefono => "Teléfono",
            Self::Fecha => "Fecha",
            Self::Hora => "Hora",
            Self::Lista => "Lista",
            Self::Notas => "Notas",
            Self::Booleano => "Sí/No (Booleano)",
            Self::Oculto => "Oculto",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tipo_campo_serializacion() {
        let t = TipoCampo::Booleano;
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"BOOLEANO\"");
    }
}
