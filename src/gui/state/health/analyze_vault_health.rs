/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Analizador integral de Salud de la Bóveda (puntuación 0-100 y desglose en 6 pestañas).

use std::collections::HashMap;
use slint::SharedString;

use crate::crypto::medir_fuerza_contrasena;
use crate::gui::state::filter_engine::subtitulo_segun_tipo;
use crate::gui::state::health::common_passwords_check::es_contrasena_comun;
use crate::gui::state::health::health_tab_kind::PestanaSalud;
use crate::gui::HealthIssueItem;
use crate::models::ContenidoBoveda;

pub fn analizar_salud_boveda(
    boveda: &ContenidoBoveda,
    pestana_str: &str,
) -> (i32, [i32; 6], Vec<HealthIssueItem>) {
    let pestana = PestanaSalud::desde_str(pestana_str);
    let ahora_ms = chrono::Utc::now().timestamp_millis();
    let umbral_antigua_ms: i64 = 180 * 86_400_000; // 180 días

    let activas: Vec<_> = boveda
        .entradas
        .iter()
        .filter(|e| e.eliminada_en == 0)
        .collect();

    let claves: Vec<_> = activas
        .iter()
        .copied()
        .filter(|e| e.tipo == crate::models::TipoEntrada::Login && !e.contrasena.trim().is_empty() && !e.ignorada_en_salud)
        .collect();

    // Contabilizar repeticiones de contraseñas no vacías en entradas tipo LOGIN no ignoradas
    let mut mapa_repetidas: HashMap<String, usize> = HashMap::new();
    for e in &claves {
        *mapa_repetidas.entry(e.contrasena.clone()).or_insert(0) += 1;
    }

    let mut items_caducadas = Vec::new();
    let mut items_repetidas = Vec::new();
    let mut items_comunes = Vec::new();
    let mut items_debiles = Vec::new();
    let mut items_antiguas = Vec::new();
    let mut items_ignoradas = Vec::new();

    let total_evaluables = claves.len();
    let mut penalizacion_acumulada = 0i32;

    for e in &activas {
        if e.ignorada_en_salud {
            let sub = subtitulo_segun_tipo(e);
            items_ignoradas.push(HealthIssueItem {
                id: SharedString::from(&e.id),
                titulo: SharedString::from(&e.titulo),
                subtitulo: SharedString::from(&sub),
                categoria_riesgo: SharedString::from("IGNORADA"),
                detalle_riesgo: SharedString::from("Excluida manualmente de la auditoría de salud"),
                severidad: SharedString::from("BAJA"),
                ignorada: true,
            });
        }
    }

    for e in &claves {
        let sub = subtitulo_segun_tipo(e);

        // 1. Caducadas
        if let Some(exp) = e.fecha_caducidad {
            if exp > 0 && exp <= ahora_ms {
                penalizacion_acumulada += 15;
                items_caducadas.push(HealthIssueItem {
                    id: SharedString::from(&e.id),
                    titulo: SharedString::from(&e.titulo),
                    subtitulo: SharedString::from(&sub),
                    categoria_riesgo: SharedString::from("CADUCADA"),
                    detalle_riesgo: SharedString::from("La fecha de caducidad de esta credencial ha vencido"),
                    severidad: SharedString::from("ALTA"),
                    ignorada: false,
                });
            }
        }

        // 2. Repetidas
        let repeticiones = mapa_repetidas.get(&e.contrasena).copied().unwrap_or(0);
        if repeticiones > 1 {
            penalizacion_acumulada += 10;
            items_repetidas.push(HealthIssueItem {
                id: SharedString::from(&e.id),
                titulo: SharedString::from(&e.titulo),
                subtitulo: SharedString::from(&sub),
                categoria_riesgo: SharedString::from("REPETIDA"),
                detalle_riesgo: SharedString::from(format!(
                    "Reutilizada en {repeticiones} cuentas de la bóveda"
                )),
                severidad: SharedString::from("ALTA"),
                ignorada: false,
            });
        }

        // 3. Comunes
        if es_contrasena_comun(&e.contrasena) {
            penalizacion_acumulada += 20;
            items_comunes.push(HealthIssueItem {
                id: SharedString::from(&e.id),
                titulo: SharedString::from(&e.titulo),
                subtitulo: SharedString::from(&sub),
                categoria_riesgo: SharedString::from("COMUN"),
                detalle_riesgo: SharedString::from("Contraseña presente en diccionarios de filtraciones comunes"),
                severidad: SharedString::from("CRITICA"),
                ignorada: false,
            });
        }

        // 4. Débiles (longitud < 8 o puntuación fuerza <= 1 idéntico a Android con zxcvbn)
        let fuerza = medir_fuerza_contrasena(&e.contrasena);
        if e.contrasena.chars().count() < 8 || fuerza.puntuacion <= 1 {
            penalizacion_acumulada += 12;
            items_debiles.push(HealthIssueItem {
                id: SharedString::from(&e.id),
                titulo: SharedString::from(&e.titulo),
                subtitulo: SharedString::from(&sub),
                categoria_riesgo: SharedString::from("DEBIL"),
                detalle_riesgo: SharedString::from(format!(
                    "Longitud {} caracteres · Nivel: {}",
                    e.contrasena.chars().count(),
                    fuerza.etiqueta
                )),
                severidad: SharedString::from("MEDIA"),
                ignorada: false,
            });
        }

        // 5. Antiguas (> 180 días sin modificar)
        let ref_ts = if e.modificada_en > 0 { e.modificada_en } else { e.creada_en };
        if ref_ts > 0 && (ahora_ms - ref_ts) > umbral_antigua_ms {
            let dias = (ahora_ms - ref_ts) / 86_400_000;
            penalizacion_acumulada += 4;
            items_antiguas.push(HealthIssueItem {
                id: SharedString::from(&e.id),
                titulo: SharedString::from(&e.titulo),
                subtitulo: SharedString::from(&sub),
                categoria_riesgo: SharedString::from("ANTIGUA"),
                detalle_riesgo: SharedString::from(format!("Sin rotar desde hace {dias} días")),
                severidad: SharedString::from("BAJA"),
                ignorada: false,
            });
        }
    }

    let conteos = [
        items_caducadas.len() as i32,
        items_repetidas.len() as i32,
        items_comunes.len() as i32,
        items_debiles.len() as i32,
        items_antiguas.len() as i32,
        items_ignoradas.len() as i32,
    ];

    let puntuacion = if total_evaluables == 0 {
        100
    } else {
        let ratio = (penalizacion_acumulada as f32) / (total_evaluables.max(1) as f32);
        (100.0 - (ratio * 3.2)).round().clamp(0.0, 100.0) as i32
    };

    let lista_activa = match pestana {
        PestanaSalud::Caducadas => items_caducadas,
        PestanaSalud::Repetidas => items_repetidas,
        PestanaSalud::Comunes => items_comunes,
        PestanaSalud::Debiles => items_debiles,
        PestanaSalud::Antiguas => items_antiguas,
        PestanaSalud::Ignoradas => items_ignoradas,
    };

    (puntuacion, conteos, lista_activa)
}
