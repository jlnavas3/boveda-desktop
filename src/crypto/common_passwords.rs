/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Verificación soberana de contraseñas filtradas y comunes sin red.

use std::collections::HashSet;
use std::sync::OnceLock;

static CONTRASENAS_COMUNES: OnceLock<HashSet<&'static str>> = OnceLock::new();

/// Comprueba si una contraseña está entre las más comunes conocidas.
pub fn es_contrasena_comun(contrasena: &str) -> bool {
    if contrasena.trim().is_empty() {
        return false;
    }
    let lista = CONTRASENAS_COMUNES.get_or_init(|| {
        let texto = include_str!("common_passwords.txt");
        texto.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect()
    });
    lista.contains(contrasena.trim().to_lowercase().as_str())
}
