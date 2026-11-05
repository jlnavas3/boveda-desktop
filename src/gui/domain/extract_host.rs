/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Extractor desacoplado del host limpio de una URL o dominio (idéntico a `Dominios.host()` de Android).

pub fn extraer_host_limpio(entrada: &str) -> String {
    let texto = entrada.trim().to_lowercase();
    if texto.is_empty() {
        return String::new();
    }

    let sin_esquema = if let Some(idx) = texto.find("://") {
        &texto[idx + 3..]
    } else {
        &texto
    };

    let sin_ruta = sin_esquema
        .split('/')
        .next()
        .unwrap_or("")
        .split('?')
        .next()
        .unwrap_or("")
        .split('#')
        .next()
        .unwrap_or("");

    let sin_credenciales = sin_ruta.rsplit('@').next().unwrap_or(sin_ruta);
    let sin_puerto = sin_credenciales.split(':').next().unwrap_or(sin_credenciales);

    sin_puerto.strip_prefix("www.").unwrap_or(sin_puerto).trim().to_string()
}
