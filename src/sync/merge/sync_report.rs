/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Informe determinista y estructurado del balance de sincronización P2P.
//! Emite en un único pase el resumen de cambios sin exponer credenciales sensibles.
//! Espejo canónico de `InformeSincronizacion.kt` en Android.

use serde::{Deserialize, Serialize};

use crate::sync::merge::diff_mask::MascaraCampos;

/// Clasificación de la transición de ciclo de vida de una entrada durante la sincronización.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TipoCambioSync {
    Nueva,
    Actualizada,
    EnviadaAPapelera,
    RestauradaDePapelera,
}

/// Detalle individual de una entrada afectada por la sincronización con su máscara de diff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemCambioSync {
    pub id: String,
    pub titulo: String,
    pub tipo_cambio: TipoCambioSync,
    pub mascara: MascaraCampos,
}

/// Informe determinista y estructurado del resultado de una sesión de sincronización P2P.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InformeSincronizacion {
    pub nuevas: Vec<ItemCambioSync>,
    pub actualizadas: Vec<ItemCambioSync>,
    pub papelera: Vec<ItemCambioSync>,
    pub sin_cambios_count: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dispositivo_remoto: Option<String>,
}

impl InformeSincronizacion {
    /// Total de entradas modificadas, agregadas o alteradas en papelera.
    pub fn total_modificadas(&self) -> usize {
        self.nuevas.len() + self.actualizadas.len() + self.papelera.len()
    }

    /// Retorna `true` si al menos una entrada fue modificada o añadida.
    pub fn hay_cambios(&self) -> bool {
        self.total_modificadas() > 0
    }

    /// Construye un resumen seguro y conciso de una línea para el Registro de Eventos y Diagnósticos.
    pub fn texto_resumen(&self) -> String {
        let prefijo = match &self.dispositivo_remoto {
            Some(d) if !d.trim().is_empty() => format!("con {d}: "),
            _ => String::new(),
        };

        if !self.hay_cambios() {
            return format!(
                "{prefijo}Bóvedas idénticas (sin cambios, {} al día)",
                self.sin_cambios_count
            );
        }

        let mut partes = Vec::new();
        if !self.nuevas.is_empty() {
            let n = self.nuevas.len();
            let titulos: Vec<String> = self
                .nuevas
                .iter()
                .take(3)
                .map(|it| format!("«{}»", it.titulo))
                .collect();
            let titulos_str = titulos.join(", ");
            let mas = if n > 3 {
                format!(" y {} más", n - 3)
            } else {
                String::new()
            };
            partes.push(format!("{n} nueva(s) ({titulos_str}{mas})"));
        }

        if !self.actualizadas.is_empty() {
            let n = self.actualizadas.len();
            let titulos: Vec<String> = self
                .actualizadas
                .iter()
                .take(3)
                .map(|it| format!("«{}»", it.titulo))
                .collect();
            let titulos_str = titulos.join(", ");
            let mas = if n > 3 {
                format!(" y {} más", n - 3)
            } else {
                String::new()
            };
            partes.push(format!("{n} actualizada(s) ({titulos_str}{mas})"));
        }

        if !self.papelera.is_empty() {
            let n = self.papelera.len();
            let titulos: Vec<String> = self
                .papelera
                .iter()
                .take(3)
                .map(|it| format!("«{}»", it.titulo))
                .collect();
            let titulos_str = titulos.join(", ");
            let mas = if n > 3 {
                format!(" y {} más", n - 3)
            } else {
                String::new()
            };
            partes.push(format!("{n} en papelera ({titulos_str}{mas})"));
        }

        if self.sin_cambios_count > 0 {
            partes.push(format!("{} sin cambios", self.sin_cambios_count));
        }

        format!("{prefijo}{}", partes.join("; "))
    }
}
