/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Tipos de filtro de detección de duplicados (idéntico a ModelosFiltroDuplicados.kt en Android).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FiltroDuplicados {
    Todos,
    Identicos,
    Passkey,
    Totp,
    AppsAndroid,
    SitiosWeb,
    MismaCuenta,
    Variantes,
}

impl FiltroDuplicados {
    pub fn desde_str(valor: &str) -> Self {
        match valor.trim().to_uppercase().as_str() {
            "IDENTICOS" => Self::Identicos,
            "PASSKEY" => Self::Passkey,
            "TOTP" => Self::Totp,
            "APPS_ANDROID" => Self::AppsAndroid,
            "SITIOS_WEB" => Self::SitiosWeb,
            "MISMA_CUENTA" => Self::MismaCuenta,
            "VARIANTES" => Self::Variantes,
            _ => Self::Todos,
        }
    }
}
