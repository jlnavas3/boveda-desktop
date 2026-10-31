/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Configuración local del cliente de escritorio de Bóveda Local.
//! Persistida en formato TOML en los directorios estándar del sistema operativo (XDG / AppData).

use crate::crypto::BvdaError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfigDesktop {
    #[serde(default)]
    pub ruta_boveda: Option<PathBuf>,
    #[serde(default = "default_tema")]
    pub tema: String,
    #[serde(default = "default_true")]
    pub limpiar_portapapeles_auto: bool,
    #[serde(default = "default_segundos_portapapeles")]
    pub segundos_limpieza_portapapeles: u32,
    #[serde(default = "default_densidad")]
    pub densidad_interfaz: String,
    #[serde(default = "default_true")]
    pub confirmar_salida: bool,
}

fn default_true() -> bool {
    true
}

fn default_tema() -> String {
    "sistema".into()
}

fn default_segundos_portapapeles() -> u32 {
    30
}

fn default_densidad() -> String {
    "predeterminada".into()
}

impl Default for AppConfigDesktop {
    fn default() -> Self {
        Self {
            ruta_boveda: None,
            tema: default_tema(),
            limpiar_portapapeles_auto: true,
            segundos_limpieza_portapapeles: default_segundos_portapapeles(),
            densidad_interfaz: default_densidad(),
            confirmar_salida: true,
        }
    }
}

impl AppConfigDesktop {
    /// Devuelve la ruta canónica del archivo de configuración TOML según el sistema operativo.
    pub fn ruta_archivo() -> PathBuf {
        let base = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("boveda-local");
        base.join("config.toml")
    }

    /// Carga la configuración desde el disco o retorna la predeterminada si no existe.
    pub fn cargar() -> Result<Self, BvdaError> {
        Self::cargar_desde(&Self::ruta_archivo())
    }

    /// Carga la configuración desde una ruta arbitraria.
    pub fn cargar_desde(ruta: &Path) -> Result<Self, BvdaError> {
        if !ruta.exists() {
            return Ok(Self::default());
        }
        let contenido = fs::read_to_string(ruta)?;
        let config: Self = toml::from_str(&contenido)
            .map_err(|e| BvdaError::ErrorConfig(format!("Error al parsear TOML: {}", e)))?;
        Ok(config)
    }

    /// Guarda la configuración actual en la ruta canónica.
    pub fn guardar(&self) -> Result<(), BvdaError> {
        self.guardar_en(&Self::ruta_archivo())
    }

    /// Guarda la configuración en una ruta específica.
    pub fn guardar_en(&self, ruta: &Path) -> Result<(), BvdaError> {
        if let Some(padre) = ruta.parent() {
            fs::create_dir_all(padre)?;
        }
        let toml_str = toml::to_string_pretty(self)
            .map_err(|e| BvdaError::ErrorConfig(format!("Error al serializar TOML: {}", e)))?;
        fs::write(ruta, toml_str)?;
        Ok(())
    }

    /// Consulta el valor de una clave por nombre.
    pub fn get_valor(&self, clave: &str) -> Option<String> {
        match clave {
            "ruta_boveda" => self.ruta_boveda.as_ref().map(|p| p.to_string_lossy().to_string()),
            "tema" => Some(self.tema.clone()),
            "limpiar_portapapeles_auto" => Some(self.limpiar_portapapeles_auto.to_string()),
            "segundos_limpieza_portapapeles" => Some(self.segundos_limpieza_portapapeles.to_string()),
            "densidad_interfaz" => Some(self.densidad_interfaz.clone()),
            "confirmar_salida" => Some(self.confirmar_salida.to_string()),
            _ => None,
        }
    }

    /// Modifica el valor de una clave por nombre.
    pub fn set_valor(&mut self, clave: &str, valor: &str) -> Result<(), BvdaError> {
        match clave {
            "ruta_boveda" => {
                self.ruta_boveda = if valor.trim().is_empty() {
                    None
                } else {
                    Some(PathBuf::from(valor))
                };
            }
            "tema" => {
                self.tema = valor.to_string();
            }
            "limpiar_portapapeles_auto" => {
                self.limpiar_portapapeles_auto = valor.parse().map_err(|_| {
                    BvdaError::ErrorConfig("Valor booleano esperado (true/false)".into())
                })?;
            }
            "segundos_limpieza_portapapeles" => {
                self.segundos_limpieza_portapapeles = valor.parse().map_err(|_| {
                    BvdaError::ErrorConfig("Valor numérico entero esperado".into())
                })?;
            }
            "densidad_interfaz" => {
                self.densidad_interfaz = valor.to_string();
            }
            "confirmar_salida" => {
                self.confirmar_salida = valor.parse().map_err(|_| {
                    BvdaError::ErrorConfig("Valor booleano esperado (true/false)".into())
                })?;
            }
            _ => {
                return Err(BvdaError::ErrorConfig(format!("Clave desconocida: '{}'", clave)));
            }
        }
        Ok(())
    }

    /// Lista todas las claves y sus valores actuales.
    pub fn listar_valores(&self) -> Vec<(String, String)> {
        vec![
            (
                "ruta_boveda".into(),
                self.ruta_boveda
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "<no configurada>".into()),
            ),
            ("tema".into(), self.tema.clone()),
            (
                "limpiar_portapapeles_auto".into(),
                self.limpiar_portapapeles_auto.to_string(),
            ),
            (
                "segundos_limpieza_portapapeles".into(),
                self.segundos_limpieza_portapapeles.to_string(),
            ),
            ("densidad_interfaz".into(), self.densidad_interfaz.clone()),
            ("confirmar_salida".into(), self.confirmar_salida.to_string()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_roundtrip() {
        let temp_dir = std::env::temp_dir().join(format!("bvda-cfg-{}", std::process::id()));
        let ruta = temp_dir.join("config.toml");

        let config = AppConfigDesktop {
            tema: "oscuro".into(),
            segundos_limpieza_portapapeles: 45,
            ruta_boveda: Some(PathBuf::from("/home/user/mi_boveda.bvda")),
            ..Default::default()
        };

        config.guardar_en(&ruta).expect("debe guardar toml");

        let cargada = AppConfigDesktop::cargar_desde(&ruta).expect("debe cargar toml");
        assert_eq!(config, cargada);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_get_set_valores() {
        let mut config = AppConfigDesktop::default();
        config.set_valor("tema", "claro").unwrap();
        assert_eq!(config.get_valor("tema").as_deref(), Some("claro"));

        config.set_valor("segundos_limpieza_portapapeles", "15").unwrap();
        assert_eq!(config.segundos_limpieza_portapapeles, 15);

        assert!(config.set_valor("clave_invalida", "val").is_err());
    }
}
