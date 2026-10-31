/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Gestor dinámico del Laboratorio de Temas y Paletas de Acento.
//! Reproduce con fidelidad absoluta los modos y las 19 paletas canónicas de Bóveda Local Android.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ModoTema {
    #[default]
    Oscuro,
    Claro,
    Amoled,
    Sistema,
}

impl ModoTema {
    pub fn desde_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "claro" | "light" => Self::Claro,
            "amoled" | "negro" => Self::Amoled,
            "sistema" | "system" => Self::Sistema,
            _ => Self::Oscuro,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Oscuro => "oscuro",
            Self::Claro => "claro",
            Self::Amoled => "amoled",
            Self::Sistema => "sistema",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaletaAcentoInfo {
    pub clave: &'static str,
    pub nombre: &'static str,
    pub hex_base_oscura: &'static str,
    pub hex_base_clara: &'static str,
    pub hex_fuerte_oscura: &'static str,
    pub hex_fuerte_clara: &'static str,
}

pub const PALETAS_ACENTO_OFICIALES: &[PaletaAcentoInfo] = &[
    PaletaAcentoInfo { clave: "menta", nombre: "Menta", hex_base_oscura: "#57E6B4", hex_base_clara: "#0D9488", hex_fuerte_oscura: "#23C08D", hex_fuerte_clara: "#0F766E" },
    PaletaAcentoInfo { clave: "azul", nombre: "Azul", hex_base_oscura: "#6FA8FF", hex_base_clara: "#2563EB", hex_fuerte_oscura: "#3D7EFF", hex_fuerte_clara: "#1D4ED8" },
    PaletaAcentoInfo { clave: "rosa", nombre: "Rosa", hex_base_oscura: "#FF8FCB", hex_base_clara: "#DB2777", hex_fuerte_oscura: "#FF5FA8", hex_fuerte_clara: "#BE185D" },
    PaletaAcentoInfo { clave: "violeta", nombre: "Violeta", hex_base_oscura: "#B98BFF", hex_base_clara: "#7C3AED", hex_fuerte_oscura: "#8C5CFF", hex_fuerte_clara: "#6D28D9" },
    PaletaAcentoInfo { clave: "rojo", nombre: "Rojo", hex_base_oscura: "#F44336", hex_base_clara: "#DC2626", hex_fuerte_oscura: "#D32F2F", hex_fuerte_clara: "#B91C1C" },
    PaletaAcentoInfo { clave: "purpura", nombre: "Púrpura", hex_base_oscura: "#9C27B0", hex_base_clara: "#7E22CE", hex_fuerte_oscura: "#7B1FA2", hex_fuerte_clara: "#6B21A8" },
    PaletaAcentoInfo { clave: "purpura_oscuro", nombre: "Púrpura oscuro", hex_base_oscura: "#673AB7", hex_base_clara: "#5B21B6", hex_fuerte_oscura: "#512DA8", hex_fuerte_clara: "#4C1D95" },
    PaletaAcentoInfo { clave: "indigo", nombre: "Índigo", hex_base_oscura: "#3F51B5", hex_base_clara: "#3730A3", hex_fuerte_oscura: "#303F9F", hex_fuerte_clara: "#312E81" },
    PaletaAcentoInfo { clave: "celeste", nombre: "Celeste", hex_base_oscura: "#03A9F4", hex_base_clara: "#0284C7", hex_fuerte_oscura: "#0288D1", hex_fuerte_clara: "#0369A1" },
    PaletaAcentoInfo { clave: "cian", nombre: "Cian", hex_base_oscura: "#00BCD4", hex_base_clara: "#0891B2", hex_fuerte_oscura: "#0097A7", hex_fuerte_clara: "#0E7490" },
    PaletaAcentoInfo { clave: "verde", nombre: "Verde", hex_base_oscura: "#4CAF50", hex_base_clara: "#16A34A", hex_fuerte_oscura: "#388E3C", hex_fuerte_clara: "#15803D" },
    PaletaAcentoInfo { clave: "verde_claro", nombre: "Verde claro", hex_base_oscura: "#8BC34A", hex_base_clara: "#4D7C0F", hex_fuerte_oscura: "#689F38", hex_fuerte_clara: "#3F6212" },
    PaletaAcentoInfo { clave: "lima", nombre: "Lima", hex_base_oscura: "#CDDC39", hex_base_clara: "#4D7C0F", hex_fuerte_oscura: "#AFB42B", hex_fuerte_clara: "#3F6212" },
    PaletaAcentoInfo { clave: "amarillo", nombre: "Amarillo", hex_base_oscura: "#FFEB3B", hex_base_clara: "#B45309", hex_fuerte_oscura: "#FBC02D", hex_fuerte_clara: "#92400E" },
    PaletaAcentoInfo { clave: "naranja", nombre: "Naranja", hex_base_oscura: "#FF9800", hex_base_clara: "#C2410C", hex_fuerte_oscura: "#F57C00", hex_fuerte_clara: "#9A3412" },
    PaletaAcentoInfo { clave: "naranja_oscuro", nombre: "Naranja oscuro", hex_base_oscura: "#FF5722", hex_base_clara: "#C2410C", hex_fuerte_oscura: "#E64A19", hex_fuerte_clara: "#9A3412" },
    PaletaAcentoInfo { clave: "marron", nombre: "Marrón", hex_base_oscura: "#8D6E63", hex_base_clara: "#5D4037", hex_fuerte_oscura: "#6D4C41", hex_fuerte_clara: "#4E342E" },
    PaletaAcentoInfo { clave: "gris", nombre: "Gris", hex_base_oscura: "#B0BEC5", hex_base_clara: "#475569", hex_fuerte_oscura: "#90A4AE", hex_fuerte_clara: "#334155" },
    PaletaAcentoInfo { clave: "gris_azulado", nombre: "Gris azulado", hex_base_oscura: "#90A4AE", hex_base_clara: "#334155", hex_fuerte_oscura: "#78909C", hex_fuerte_clara: "#1E293B" },
];

pub fn obtener_paleta_acento(clave: &str) -> &'static PaletaAcentoInfo {
    let limpia = clave.trim().to_lowercase();
    PALETAS_ACENTO_OFICIALES
        .iter()
        .find(|p| p.clave == limpia)
        .unwrap_or(&PALETAS_ACENTO_OFICIALES[0]) // Menta por defecto
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColoresTemaResueltos {
    pub es_oscuro: bool,
    pub background: &'static str,
    pub surface: &'static str,
    pub surface_hover: &'static str,
    pub surface_selected: &'static str,
    pub border: &'static str,
    pub border_focus: &'static str,
    pub text_primary: &'static str,
    pub text_secondary: &'static str,
    pub text_muted: &'static str,
    pub accent_primary: &'static str,
    pub accent_hover: &'static str,
    pub danger: &'static str,
}

pub fn resolver_tema(
    modo: ModoTema,
    clave_acento: &str,
    sistema_es_oscuro: bool,
) -> ColoresTemaResueltos {
    let es_oscuro = match modo {
        ModoTema::Oscuro | ModoTema::Amoled => true,
        ModoTema::Claro => false,
        ModoTema::Sistema => sistema_es_oscuro,
    };

    let paleta = obtener_paleta_acento(clave_acento);
    let (accent_primary, accent_hover) = if es_oscuro {
        (paleta.hex_base_oscura, paleta.hex_fuerte_oscura)
    } else {
        (paleta.hex_base_clara, paleta.hex_fuerte_clara)
    };

    if modo == ModoTema::Amoled {
        ColoresTemaResueltos {
            es_oscuro: true,
            background: "#000000",
            surface: "#0A0D12",
            surface_hover: "#151A21",
            surface_selected: "#1B222C",
            border: "#21262D",
            border_focus: accent_primary,
            text_primary: "#FFFFFF",
            text_secondary: "#A0AEC0",
            text_muted: "#718096",
            accent_primary,
            accent_hover,
            danger: "#EF4444",
        }
    } else if es_oscuro {
        ColoresTemaResueltos {
            es_oscuro: true,
            background: "#0D1117",
            surface: "#161B22",
            surface_hover: "#21262D",
            surface_selected: "#1F2937",
            border: "#30363D",
            border_focus: accent_primary,
            text_primary: "#F0F6FC",
            text_secondary: "#8B949E",
            text_muted: "#6E7681",
            accent_primary,
            accent_hover,
            danger: "#EF4444",
        }
    } else {
        ColoresTemaResueltos {
            es_oscuro: false,
            background: "#F6F8FA",
            surface: "#FFFFFF",
            surface_hover: "#EAEFF5",
            surface_selected: "#DCE5F0",
            border: "#D0D7DE",
            border_focus: accent_primary,
            text_primary: "#1F2328",
            text_secondary: "#656D76",
            text_muted: "#8C959F",
            accent_primary,
            accent_hover,
            danger: "#CF222E",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diecinueve_paletas_acento() {
        assert_eq!(PALETAS_ACENTO_OFICIALES.len(), 19);
        let menta = obtener_paleta_acento("menta");
        assert_eq!(menta.nombre, "Menta");
        let azul = obtener_paleta_acento("azul");
        assert_eq!(azul.nombre, "Azul");
    }

    #[test]
    fn test_resolucion_tema_amoled_y_claro() {
        let amoled = resolver_tema(ModoTema::Amoled, "rosa", true);
        assert_eq!(amoled.background, "#000000");
        assert_eq!(amoled.accent_primary, "#FF8FCB");

        let claro = resolver_tema(ModoTema::Claro, "menta", true);
        assert_eq!(claro.background, "#F6F8FA");
        assert_eq!(claro.accent_primary, "#0D9488");
        assert!(!claro.es_oscuro);
    }
}
