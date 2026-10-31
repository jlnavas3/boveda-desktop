/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Conversor y manipulador atómico de reglas de puertos locales y subdominios para Slint.

use slint::{ModelRc, VecModel};
#[cfg(test)]
use slint::Model;
use crate::gui::{PortRuleItem, PortRuleRow, SubdomainRuleItem, SubdomainRuleRow};

/// Ancho de fallback para cuando no se especifica explícitamente el ancho del panel.
pub const ANCHO_FALLBACK_PANEL: f32 = 720.0;

/// Parsea una cadena multilínea de puertos y agrupa los elementos en filas adaptadas dinámicamente al ancho disponible.
pub fn parse_puertos_filas_con_ancho(puertos_str: &str, ancho_disponible: f32) -> Vec<PortRuleRow> {
    let items = parse_puertos_items(puertos_str);
    let ancho_efectivo = (ancho_disponible - 40.0).max(250.0);
    let mut rows = Vec::new();
    let mut current_row: Vec<PortRuleItem> = Vec::new();
    let mut current_w: f32 = 0.0;

    for item in items {
        // Estimación de ancho: 46px (padding + botón x + separador) + ~7.2px/char + 8px spacing
        let item_w = 46.0 + (item.texto.chars().count() as f32 * 7.2) + 8.0;
        if !current_row.is_empty() && current_w + item_w > ancho_efectivo {
            rows.push(PortRuleRow {
                items: ModelRc::new(VecModel::from(std::mem::take(&mut current_row))),
            });
            current_w = 0.0;
        }
        current_w += item_w;
        current_row.push(item);
    }

    if !current_row.is_empty() {
        rows.push(PortRuleRow {
            items: ModelRc::new(VecModel::from(current_row)),
        });
    }

    rows
}

/// Parsea una cadena multilínea de puertos usando el ancho de fallback predeterminado.
pub fn parse_puertos_filas(puertos_str: &str) -> Vec<PortRuleRow> {
    parse_puertos_filas_con_ancho(puertos_str, ANCHO_FALLBACK_PANEL)
}

/// Parsea una cadena de subdominios y agrupa los elementos en filas adaptadas dinámicamente al ancho disponible.
pub fn parse_subdominios_filas_con_ancho(subdominios_str: &str, ancho_disponible: f32) -> Vec<SubdomainRuleRow> {
    let items = parse_subdominios_items(subdominios_str);
    let ancho_efectivo = (ancho_disponible - 40.0).max(250.0);
    let mut rows = Vec::new();
    let mut current_row: Vec<SubdomainRuleItem> = Vec::new();
    let mut current_w: f32 = 0.0;

    for item in items {
        let item_w = 46.0 + (item.texto.chars().count() as f32 * 7.2) + 8.0;
        if !current_row.is_empty() && current_w + item_w > ancho_efectivo {
            rows.push(SubdomainRuleRow {
                items: ModelRc::new(VecModel::from(std::mem::take(&mut current_row))),
            });
            current_w = 0.0;
        }
        current_w += item_w;
        current_row.push(item);
    }

    if !current_row.is_empty() {
        rows.push(SubdomainRuleRow {
            items: ModelRc::new(VecModel::from(current_row)),
        });
    }

    rows
}

/// Parsea una cadena de subdominios usando el ancho de fallback predeterminado.
pub fn parse_subdominios_filas(subdominios_str: &str) -> Vec<SubdomainRuleRow> {
    parse_subdominios_filas_con_ancho(subdominios_str, ANCHO_FALLBACK_PANEL)
}

/// Parsea una cadena multilínea de puertos ("80: HTTP\n443: HTTPS") en una lista de `PortRuleItem`.
pub fn parse_puertos_items(puertos_str: &str) -> Vec<PortRuleItem> {
    puertos_str
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }
            let id = if let Some((puerto, _)) = trimmed.split_once(':') {
                puerto.trim().to_string()
            } else {
                trimmed.to_string()
            };
            Some(PortRuleItem {
                id: id.into(),
                texto: trimmed.into(),
            })
        })
        .collect()
}

/// Parsea una cadena de subdominios separados por comas o saltos de línea en una lista de `SubdomainRuleItem`.
pub fn parse_subdominios_items(subdominios_str: &str) -> Vec<SubdomainRuleItem> {
    subdominios_str
        .split([',', '\n'])
        .filter_map(|part| {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                return None;
            }
            Some(SubdomainRuleItem {
                id: trimmed.into(),
                texto: trimmed.into(),
            })
        })
        .collect()
}

/// Añade una regla de puerto a la cadena multilínea existente.
pub fn agregar_puerto_a_str(actual: &str, puerto: &str, nombre: &str) -> String {
    let puerto_trim = puerto.trim();
    let nombre_trim = nombre.trim();
    if puerto_trim.is_empty() || nombre_trim.is_empty() {
        return actual.to_string();
    }

    let mut lineas: Vec<String> = actual
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

    let prefix = format!("{puerto_trim}:");
    lineas.retain(|l| !l.starts_with(&prefix));
    lineas.push(format!("{puerto_trim}: {nombre_trim}"));
    lineas.join("\n")
}

/// Elimina una regla de puerto por su identificador/número de puerto.
pub fn eliminar_puerto_de_str(actual: &str, id_puerto: &str) -> String {
    let id_trim = id_puerto.trim();
    let prefix = format!("{id_trim}:");
    let lineas: Vec<String> = actual
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty() && !l.starts_with(&prefix) && l != id_trim)
        .collect();
    lineas.join("\n")
}

/// Añade un prefijo de subdominio a la cadena separada por comas existente.
pub fn agregar_subdominio_a_str(actual: &str, subdominio: &str) -> String {
    let sub_trim = subdominio.trim().to_lowercase();
    if sub_trim.is_empty() {
        return actual.to_string();
    }

    let mut subs: Vec<String> = actual
        .split([',', '\n'])
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    if !subs.contains(&sub_trim) {
        subs.push(sub_trim);
    }
    subs.join(", ")
}

/// Elimina un prefijo de subdominio de la cadena separada por comas existente.
pub fn eliminar_subdominio_de_str(actual: &str, id_subdominio: &str) -> String {
    let target = id_subdominio.trim().to_lowercase();
    let subs: Vec<String> = actual
        .split([',', '\n'])
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty() && s != &target)
        .collect();
    subs.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_puertos_parsing_y_manipulacion() {
        let raw = "80: HTTP\n443: HTTPS\n8080: Traefik";
        let items = parse_puertos_items(raw);
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].id.as_str(), "80");
        assert_eq!(items[0].texto.as_str(), "80: HTTP");
        assert_eq!(items[2].id.as_str(), "8080");

        let agregado = agregar_puerto_a_str(raw, "9000", "Portainer");
        let items2 = parse_puertos_items(&agregado);
        assert_eq!(items2.len(), 4);
        assert_eq!(items2[3].texto.as_str(), "9000: Portainer");

        let eliminado = eliminar_puerto_de_str(&agregado, "443");
        let items3 = parse_puertos_items(&eliminado);
        assert_eq!(items3.len(), 3);
        assert!(!items3.iter().any(|i| i.id.as_str() == "443"));
    }

    #[test]
    fn test_subdominios_parsing_y_manipulacion() {
        let raw = "admin, app, api, blog";
        let items = parse_subdominios_items(raw);
        assert_eq!(items.len(), 4);
        assert_eq!(items[0].id.as_str(), "admin");

        let agregado = agregar_subdominio_a_str(raw, "dashboard");
        let items2 = parse_subdominios_items(&agregado);
        assert_eq!(items2.len(), 5);
        assert!(items2.iter().any(|i| i.id.as_str() == "dashboard"));

        let eliminado = eliminar_subdominio_de_str(&agregado, "api");
        let items3 = parse_subdominios_items(&eliminado);
        assert_eq!(items3.len(), 4);
        assert!(!items3.iter().any(|i| i.id.as_str() == "api"));
    }

    #[test]
    fn test_filas_agrupamiento() {
        let raw_p = "80: HTTP\n443: HTTPS\n8080: Traefik\n9000: Portainer\n53: DNS";
        // Con ancho estrecho (350px), deben agruparse en al menos 2 filas
        let filas_estrechas = parse_puertos_filas_con_ancho(raw_p, 350.0);
        assert!(filas_estrechas.len() >= 2);

        // Con ancho muy amplio (1800px), caben todos en 1 sola fila
        let filas_amplias = parse_puertos_filas_con_ancho(raw_p, 1800.0);
        assert_eq!(filas_amplias.len(), 1);
        assert_eq!(filas_amplias[0].items.row_count(), 5);

        let raw_s = "admin, app, api, blog, mail, dev, staging";
        let filas_s_amplias = parse_subdominios_filas_con_ancho(raw_s, 1800.0);
        assert_eq!(filas_s_amplias.len(), 1);
        assert_eq!(filas_s_amplias[0].items.row_count(), 7);

        let filas_s_estrechas = parse_subdominios_filas_con_ancho(raw_s, 300.0);
        assert!(filas_s_estrechas.len() >= 3);
    }
}
