/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Extractor desacoplado del dominio raíz registrable (eTLD+1) y marca del servicio.
//! Réplica fiel en Rust de `Dominios.raiz()` y `Dominios.marca()` de Android.

use crate::gui::domain::extract_host::extraer_host_limpio;

pub const SUFIJOS_PUBLICOS_COMPUESTOS: &[&str] = &[
    "gob.ec", "com.ec", "edu.ec", "org.ec", "net.ec", "med.ec", "mil.ec",
    "com.es", "org.es", "gob.es", "edu.es", "nom.es",
    "co.uk", "org.uk", "gov.uk", "me.uk", "ltd.uk", "plc.uk",
    "com.ar", "gob.ar", "org.ar", "edu.ar",
    "com.mx", "gob.mx", "org.mx", "edu.mx",
    "com.co", "gov.co", "org.co", "edu.co",
    "com.pe", "gob.pe", "org.pe", "edu.pe",
    "com.cl", "gob.cl", "org.cl", "edu.cl",
    "com.br", "gov.br", "org.br", "edu.br",
    "co.jp", "ne.jp", "ac.jp", "go.jp",
    "co.kr", "or.kr", "go.kr",
    "com.au", "net.au", "org.au", "gov.au",
    "withgoogle.com", "github.io", "gitlab.io",
];

pub fn extraer_raiz_y_marca(entrada: &str) -> (String, String) {
    let host = extraer_host_limpio(entrada);
    if host.is_empty() || !host.contains('.') {
        let m = host.clone();
        return (host, m);
    }

    // Si es IP numérica directa (ej. 192.168.1.1)
    if host.split('.').all(|p| p.parse::<u8>().is_ok()) {
        return (host.clone(), host);
    }

    let partes: Vec<&str> = host.split('.').collect();
    if partes.len() <= 1 {
        return (host.clone(), host);
    }

    // Verificar si termina en algún sufijo compuesto conocido
    for sufijo in SUFIJOS_PUBLICOS_COMPUESTOS {
        if host == *sufijo {
            return (host.clone(), host);
        }
        let punto_sufijo = format!(".{}", sufijo);
        if host.ends_with(&punto_sufijo) {
            let sin_sufijo = &host[..host.len() - punto_sufijo.len()];
            let marca = sin_sufijo.split('.').next_back().unwrap_or(sin_sufijo);
            let raiz = format!("{}.{}", marca, sufijo);
            return (raiz, marca.to_string());
        }
    }

    // TLD simple estándar (ej. github.com, facebook.com, utpl.edu, etc.)
    if partes.len() >= 2 {
        let n = partes.len();
        let marca = partes[n - 2];
        let raiz = format!("{}.{}", partes[n - 2], partes[n - 1]);
        return (raiz, marca.to_string());
    }

    (host.clone(), host)
}
