/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Extractor canónico desacoplado del nombre de servicio para una entrada.
//! Implementa fielmente las reglas de `NormalizadorTitulosSitios.kt` de Android.

use crate::gui::domain::{extraer_host_limpio, extraer_raiz_y_marca, obtener_marca_oficial};

pub fn extraer_nombre_servicio_canonico(url: &str, titulo: &str) -> String {
    let url_limpia = url.trim();
    if !url_limpia.is_empty() {
        let host = extraer_host_limpio(url_limpia);
        if !host.is_empty() {
            let (_, marca) = extraer_raiz_y_marca(&host);
            let marca_lower = marca.to_lowercase();
            if let Some(marca_oficial) = obtener_marca_oficial(&marca_lower) {
                return marca_oficial.to_string();
            }
            if !marca.is_empty() {
                return capitalizar_palabras(&marca);
            }
        }
    }

    let t = titulo.trim();
    if t.is_empty() {
        return "General".to_string();
    }

    // Si el título es una URL o dominio (ej. "https://github.com" o "github.com")
    if t.contains("://") || (t.contains('.') && !t.contains(' ')) {
        let host = extraer_host_limpio(t);
        let (_, marca) = extraer_raiz_y_marca(&host);
        let marca_lower = marca.to_lowercase();
        if let Some(marca_oficial) = obtener_marca_oficial(&marca_lower) {
            return marca_oficial.to_string();
        }
        if !marca.is_empty() {
            return capitalizar_palabras(&marca);
        }
    }

    // Cortar prefijos o notas en paréntesis o separadores (ej: "Google - Trabajo" -> "Google")
    let primer_token = t
        .split(['(', '-', ':', '|', '/', '·', '['])
        .next()
        .unwrap_or(t)
        .trim();

    let token_lower = primer_token.to_lowercase();
    if let Some(marca_oficial) = obtener_marca_oficial(&token_lower) {
        return marca_oficial.to_string();
    }

    capitalizar_palabras(primer_token)
}

fn capitalizar_palabras(s: &str) -> String {
    let limpio = s.replace(['-', '_'], " ");
    let mut resultado = Vec::new();
    for palabra in limpio.split_whitespace() {
        let mut chars = palabra.chars();
        if let Some(primera) = chars.next() {
            let rest: String = chars.as_str().to_lowercase();
            resultado.push(format!("{}{}", primera.to_uppercase(), rest));
        }
    }
    if resultado.is_empty() {
        s.to_string()
    } else {
        resultado.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extraccion_canonico_gob_ec_y_marcas() {
        // Casos clave de la captura del usuario:
        assert_eq!(
            extraer_nombre_servicio_canonico("https://sipostal.serviciopostal.gob.ec/", "Serviciopostal"),
            "Serviciopostal"
        );
        assert_eq!(
            extraer_nombre_servicio_canonico("https://srienlinea.sri.gob.ec/", "SRI Ecuador"),
            "SRI Ecuador"
        );
        assert_eq!(
            extraer_nombre_servicio_canonico("https://www.iess.gob.ec/", "IESS"),
            "IESS"
        );
        assert_eq!(
            extraer_nombre_servicio_canonico("https://www.facebook.com/", "Facebook"),
            "Facebook"
        );
        assert_eq!(
            extraer_nombre_servicio_canonico("https://aws.amazon.co.uk/", "Amazon"),
            "Amazon"
        );
    }
}
