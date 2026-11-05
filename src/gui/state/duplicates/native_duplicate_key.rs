/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Extractor de clave canónica para duplicados en tipos nativos especializados.
//! Réplica fiel de `ExtractorClaveDuplicadoNativo.kt` de Android.

use crate::models::{Entrada, TipoEntrada};

/// Clave de identidad nativa especializada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaveDuplicadoNativo {
    pub identificador: String,
    pub etiqueta_visual: String,
}

/// Comprueba si el tipo de entrada es especializado y debe evaluarse por identificadores estrictos.
pub fn es_tipo_especializado_duplicados(tipo: TipoEntrada) -> bool {
    matches!(
        tipo,
        TipoEntrada::Tarjeta
            | TipoEntrada::Wifi
            | TipoEntrada::CuentaBancaria
            | TipoEntrada::Identidad
            | TipoEntrada::Servidor
            | TipoEntrada::Wallet
            | TipoEntrada::Contacto
            | TipoEntrada::Nota
    )
}

/// Extrae la clave especializada si aplica.
pub fn extraer_clave_duplicado_nativo(entrada: &Entrada) -> Option<ClaveDuplicadoNativo> {
    match entrada.tipo {
        TipoEntrada::Wifi => {
            let wifi = entrada.datos_wifi.as_ref()?;
            let ssid = wifi.ssid.trim().to_lowercase();
            if ssid.is_empty() {
                return None;
            }
            Some(ClaveDuplicadoNativo {
                identificador: format!("wifi:{ssid}"),
                etiqueta_visual: format!("Wi-Fi · {}", wifi.ssid.trim()),
            })
        }
        TipoEntrada::Tarjeta => {
            let tar = entrada.datos_tarjeta.as_ref()?;
            let num: String = tar.numero.chars().filter(|c| c.is_ascii_digit()).collect();
            if num.len() < 8 {
                return None;
            }
            let ultimos = if num.len() >= 4 { &num[num.len() - 4..] } else { &num };
            let marca = if !tar.marca.trim().is_empty() {
                &tar.marca
            } else {
                "Tarjeta"
            };
            Some(ClaveDuplicadoNativo {
                identificador: format!("tarjeta:{num}"),
                etiqueta_visual: format!("{marca} · •••• {ultimos}"),
            })
        }
        TipoEntrada::CuentaBancaria => {
            let cta = entrada.datos_cuenta_bancaria.as_ref()?;
            let num_base = if !cta.cbu_clabe.trim().is_empty() {
                &cta.cbu_clabe
            } else {
                &cta.numero_cuenta
            };
            let norm: String = num_base.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
            if norm.len() < 6 {
                return None;
            }
            let ultimos = if norm.len() >= 4 { &norm[norm.len() - 4..] } else { &norm };
            let banco = if !cta.banco.trim().is_empty() { &cta.banco } else { "Cuenta bancaria" };
            Some(ClaveDuplicadoNativo {
                identificador: format!("cuenta:{}", norm.to_uppercase()),
                etiqueta_visual: format!("{banco} · •••• {ultimos}"),
            })
        }
        TipoEntrada::Identidad => {
            let doc = entrada.datos_identidad.as_ref()?;
            let num: String = doc.numero.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
            if num.is_empty() {
                return None;
            }
            let tipo_doc = if !doc.tipo_documento.trim().is_empty() {
                &doc.tipo_documento
            } else {
                "Documento"
            };
            Some(ClaveDuplicadoNativo {
                identificador: format!("identidad:{}:{}", tipo_doc.to_lowercase(), num.to_uppercase()),
                etiqueta_visual: format!("{tipo_doc} · {}", num.to_uppercase()),
            })
        }
        TipoEntrada::Servidor => {
            let srv = entrada.datos_servidor.as_ref()?;
            let host = srv.host.trim().to_lowercase();
            if host.is_empty() {
                return None;
            }
            let puerto = if !srv.puerto.trim().is_empty() {
                srv.puerto.trim()
            } else {
                "22"
            };
            let usr = if !srv.usuario.trim().is_empty() {
                srv.usuario.trim()
            } else {
                entrada.usuario.trim()
            };
            Some(ClaveDuplicadoNativo {
                identificador: format!("servidor:{host}:{puerto}:{usr}"),
                etiqueta_visual: if !usr.is_empty() {
                    format!("{usr}@{host}:{puerto}")
                } else {
                    format!("{host}:{puerto}")
                },
            })
        }
        TipoEntrada::Wallet => {
            let wal = entrada.datos_wallet.as_ref()?;
            let dir = wal.direccion_publica.trim().to_lowercase();
            if dir.is_empty() {
                return None;
            }
            let red = wal.red.trim().to_lowercase();
            let visible = if dir.len() > 10 {
                format!("{}...{}", &dir[..6], &dir[dir.len() - 4..])
            } else {
                dir.clone()
            };
            Some(ClaveDuplicadoNativo {
                identificador: format!("wallet:{red}:{dir}"),
                etiqueta_visual: format!("{} · {visible}", wal.red.trim()),
            })
        }
        _ => None,
    }
}
