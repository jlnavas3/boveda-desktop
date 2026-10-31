/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Los 10 tipos nativos de entrada soportados en la bóveda.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TipoEntrada {
    #[default]
    Login,
    Nota,
    Tarjeta,
    Wifi,
    CuentaBancaria,
    Identidad,
    Servidor,
    Wallet,
    Passkey,
    Contacto,
}

impl TipoEntrada {
    /// Nombre visual legible en español idéntico a Android.
    pub const fn etiqueta(&self) -> &'static str {
        match self {
            Self::Login => "Contraseña",
            Self::Nota => "Nota segura",
            Self::Tarjeta => "Tarjeta bancaria",
            Self::Wifi => "Red Wi-Fi",
            Self::CuentaBancaria => "Cuenta bancaria",
            Self::Identidad => "Documento de identidad",
            Self::Servidor => "Servidor / SSH",
            Self::Wallet => "Cripto Wallet",
            Self::Passkey => "Llave de acceso",
            Self::Contacto => "Contacto seguro",
        }
    }

    /// Identificador canónico en mayúsculas compatible con Slint.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Login => "LOGIN",
            Self::Nota => "NOTA_SEGURA",
            Self::Tarjeta => "TARJETA",
            Self::Wifi => "WIFI",
            Self::CuentaBancaria => "CUENTA_BANCARIA",
            Self::Identidad => "IDENTIDAD",
            Self::Servidor => "SERVIDOR",
            Self::Wallet => "WALLET",
            Self::Passkey => "PASSKEY",
            Self::Contacto => "CONTACTO",
        }
    }

    /// Parsea un identificador canónico o de Slint a TipoEntrada.
    pub fn desde_str(s: &str) -> Self {
        match s {
            "LOGIN" => Self::Login,
            "NOTA_SEGURA" | "NOTA" => Self::Nota,
            "TARJETA" => Self::Tarjeta,
            "WIFI" => Self::Wifi,
            "CUENTA_BANCARIA" => Self::CuentaBancaria,
            "IDENTIDAD" => Self::Identidad,
            "SERVIDOR" => Self::Servidor,
            "WALLET" => Self::Wallet,
            "PASSKEY" => Self::Passkey,
            "CONTACTO" => Self::Contacto,
            _ => Self::Login,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serializacion_screaming_snake_case() {
        let t = TipoEntrada::CuentaBancaria;
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"CUENTA_BANCARIA\"");

        let des: TipoEntrada = serde_json::from_str("\"CUENTA_BANCARIA\"").unwrap();
        assert_eq!(des, TipoEntrada::CuentaBancaria);
    }
}
