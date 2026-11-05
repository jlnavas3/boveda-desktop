/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Identificación y nombre del equipo Desktop para emparejamiento y sincronización P2P.
//! Espejo canónico de `IdentidadDispositivoLocal.kt` en Android.

use serde::{Deserialize, Serialize};

/// Mensaje de pre-anuncio intercambiado opcionalmente para identificar los pares antes de sincronizar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaludoP2P {
    #[serde(rename = "aliasDispositivo")]
    pub alias_dispositivo: String,
}

impl SaludoP2P {
    pub fn local() -> Self {
        Self {
            alias_dispositivo: obtener_nombre_desktop(),
        }
    }
}

/// Formatea el nombre legible del dispositivo Desktop combinando el hostname con el tipo de OS.
pub fn formatear_nombre_desktop(raw_name: &str) -> String {
    let trimmed = raw_name.trim();
    let sufijo = if cfg!(target_os = "windows") {
        "(Windows Desktop)"
    } else {
        "(Linux Desktop)"
    };

    if trimmed.is_empty() {
        if cfg!(target_os = "windows") {
            "Windows Desktop".to_string()
        } else {
            "Linux Desktop".to_string()
        }
    } else if trimmed.ends_with(sufijo) {
        trimmed.to_string()
    } else {
        let mut chars = trimmed.chars();
        let capitalizado = match chars.next() {
            None => String::new(),
            Some(primero) => primero.to_uppercase().collect::<String>() + chars.as_str(),
        };
        format!("{capitalizado} {sufijo}")
    }
}

/// Obtiene el nombre del equipo Desktop leyendo `/etc/hostname`, `/proc/sys/kernel/hostname`
/// o variables de entorno del sistema (`HOSTNAME`, `COMPUTERNAME`).
pub fn obtener_nombre_desktop() -> String {
    if let Ok(contenido) = std::fs::read_to_string("/etc/hostname") {
        let h = contenido.trim();
        if !h.is_empty() {
            return formatear_nombre_desktop(h);
        }
    }

    if let Ok(contenido) = std::fs::read_to_string("/proc/sys/kernel/hostname") {
        let h = contenido.trim();
        if !h.is_empty() {
            return formatear_nombre_desktop(h);
        }
    }

    if let Ok(h) = std::env::var("HOSTNAME") {
        let trimmed = h.trim();
        if !trimmed.is_empty() {
            return formatear_nombre_desktop(trimmed);
        }
    }

    if let Ok(h) = std::env::var("COMPUTERNAME") {
        let trimmed = h.trim();
        if !trimmed.is_empty() {
            return formatear_nombre_desktop(trimmed);
        }
    }

    formatear_nombre_desktop("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formatear_nombre_desktop() {
        let esperado_thinkpad = if cfg!(target_os = "windows") {
            "ThinkPad (Windows Desktop)"
        } else {
            "ThinkPad (Linux Desktop)"
        };
        assert_eq!(formatear_nombre_desktop("ThinkPad"), esperado_thinkpad);

        let esperado_fedor = if cfg!(target_os = "windows") {
            "Fedor (Windows Desktop)"
        } else {
            "Fedor (Linux Desktop)"
        };
        assert_eq!(formatear_nombre_desktop("fedor"), esperado_fedor);

        let esperado_vacio = if cfg!(target_os = "windows") {
            "Windows Desktop"
        } else {
            "Linux Desktop"
        };
        assert_eq!(formatear_nombre_desktop(""), esperado_vacio);
        assert_eq!(formatear_nombre_desktop("   "), esperado_vacio);

        // Sin duplicar el sufijo si ya lo incluye
        assert_eq!(
            formatear_nombre_desktop(esperado_thinkpad),
            esperado_thinkpad
        );
    }

    #[test]
    fn test_obtener_nombre_desktop_no_vacio() {
        let nombre = obtener_nombre_desktop();
        assert!(!nombre.is_empty());
        let sufijo = if cfg!(target_os = "windows") {
            "Desktop"
        } else {
            "Linux Desktop"
        };
        assert!(nombre.contains(sufijo));
    }
}
