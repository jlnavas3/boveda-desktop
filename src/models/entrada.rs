/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Modelo inmutable de entrada (credencial, tarjeta, nota, etc.).
//! Compatible al 100% con Entrada.kt en Bóveda Local Android.

use crate::models::adjunto::AdjuntoBoveda;
use crate::models::cambio_contrasena::CambioContrasena;
use crate::models::campo_personalizado::CampoPersonalizado;
use crate::models::contacto::DatosContacto;
use crate::models::cuenta_bancaria::DatosCuentaBancaria;
use crate::models::identidad_doc::DatosDocumentoIdentidad;
use crate::models::passkey::DatosPasskey;
use crate::models::servidor::DatosServidorSsh;
use crate::models::tarjeta::DatosTarjetaBancaria;
use crate::models::tipo_entrada::TipoEntrada;
use crate::models::totp_secundario::TotpSecundario;
use crate::models::wallet::DatosCriptoWallet;
use crate::models::wifi::DatosRedWifi;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entrada {
    pub id: String,
    #[serde(default)]
    pub tipo: TipoEntrada,
    #[serde(default)]
    pub titulo: String,
    #[serde(default)]
    pub usuario: String,
    #[serde(default)]
    pub contrasena: String,
    #[serde(default)]
    pub urls: Vec<String>,
    #[serde(default)]
    pub notas: String,
    #[serde(default)]
    pub secreto_totp: Option<String>,
    #[serde(default)]
    pub totp_emisor: String,
    #[serde(default = "default_totp_digitos")]
    pub totp_digitos: i32,
    #[serde(default = "default_totp_periodo")]
    pub totp_periodo: i32,
    #[serde(default = "default_totp_algoritmo")]
    pub totp_algoritmo: String,
    #[serde(default)]
    pub favorito: bool,
    #[serde(default)]
    pub anclado: bool,
    #[serde(default)]
    pub creada_en: i64,
    #[serde(default)]
    pub modificada_en: i64,
    #[serde(default)]
    pub ultimo_uso_en: i64,
    #[serde(default)]
    pub eliminada_en: i64,
    #[serde(default)]
    pub etiquetas: Vec<String>,
    #[serde(default)]
    pub historial_contrasenas: Vec<CambioContrasena>,
    #[serde(default)]
    pub passkey: Option<DatosPasskey>,
    #[serde(default)]
    pub passkeys_secundarias: Vec<DatosPasskey>,
    #[serde(default)]
    pub totps_secundarios: Vec<TotpSecundario>,
    #[serde(default)]
    pub campos_personalizados: Vec<CampoPersonalizado>,

    // Submodelos nativos fuertemente tipados
    #[serde(default)]
    pub datos_tarjeta: Option<DatosTarjetaBancaria>,
    #[serde(default)]
    pub datos_wifi: Option<DatosRedWifi>,
    #[serde(default)]
    pub datos_cuenta_bancaria: Option<DatosCuentaBancaria>,
    #[serde(default)]
    pub datos_identidad: Option<DatosDocumentoIdentidad>,
    #[serde(default)]
    pub datos_servidor: Option<DatosServidorSsh>,
    #[serde(default)]
    pub datos_wallet: Option<DatosCriptoWallet>,
    #[serde(default)]
    pub datos_contacto: Option<DatosContacto>,

    #[serde(default)]
    pub ignorada_en_salud: bool,
    #[serde(default, rename = "colecciones", alias = "categorias")]
    pub categorias: Vec<String>,
    #[serde(default, rename = "identidadId", alias = "perfilId")]
    pub perfil_id: Option<String>,
    #[serde(default)]
    pub fecha_caducidad: Option<i64>,
    #[serde(default)]
    pub adjuntos: Vec<AdjuntoBoveda>,
    #[serde(default)]
    pub titulo_personalizado: bool,

    // Metadatos esenciales de sincronización
    #[serde(default = "default_revision")]
    pub revision: u64,
    #[serde(default, skip_serializing_if = "Option::is_none", with = "hex_option_serde")]
    pub last_sync_hash: Option<[u8; 32]>,
}

fn default_revision() -> u64 {
    1
}

fn default_totp_digitos() -> i32 {
    6
}

fn default_totp_periodo() -> i32 {
    30
}

fn default_totp_algoritmo() -> String {
    "HmacSHA1".into()
}

impl Default for Entrada {
    fn default() -> Self {
        Self {
            id: String::new(),
            tipo: TipoEntrada::Login,
            titulo: String::new(),
            usuario: String::new(),
            contrasena: String::new(),
            urls: Vec::new(),
            notas: String::new(),
            secreto_totp: None,
            totp_emisor: String::new(),
            totp_digitos: default_totp_digitos(),
            totp_periodo: default_totp_periodo(),
            totp_algoritmo: default_totp_algoritmo(),
            favorito: false,
            anclado: false,
            creada_en: 0,
            modificada_en: 0,
            ultimo_uso_en: 0,
            eliminada_en: 0,
            etiquetas: Vec::new(),
            historial_contrasenas: Vec::new(),
            passkey: None,
            passkeys_secundarias: Vec::new(),
            totps_secundarios: Vec::new(),
            campos_personalizados: Vec::new(),
            datos_tarjeta: None,
            datos_wifi: None,
            datos_cuenta_bancaria: None,
            datos_identidad: None,
            datos_servidor: None,
            datos_wallet: None,
            datos_contacto: None,
            ignorada_en_salud: false,
            categorias: Vec::new(),
            perfil_id: None,
            fecha_caducidad: None,
            adjuntos: Vec::new(),
            titulo_personalizado: false,
            revision: 1,
            last_sync_hash: None,
        }
    }
}

mod hex_option_serde {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(val: &Option<[u8; 32]>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match val {
            Some(bytes) => {
                let mut s = String::with_capacity(64);
                for b in bytes {
                    use std::fmt::Write;
                    let _ = write!(s, "{:02x}", b);
                }
                serializer.serialize_str(&s)
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<[u8; 32]>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::<String>::deserialize(deserializer)?;
        match opt {
            Some(hex) => {
                if hex.len() != 64 {
                    return Err(serde::de::Error::custom(
                        "last_sync_hash debe tener 64 caracteres hexadecimales",
                    ));
                }
                let mut bytes = [0u8; 32];
                for i in 0..32 {
                    bytes[i] = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)
                        .map_err(serde::de::Error::custom)?;
                }
                Ok(Some(bytes))
            }
            None => Ok(None),
        }
    }
}

impl Entrada {
    pub fn tiene_totp(&self) -> bool {
        self.secreto_totp.as_ref().is_some_and(|s| !s.trim().is_empty())
            || !self.totps_secundarios.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entrada_serializacion_compatibilidad_android() {
        let entrada = Entrada {
            id: "e-1".into(),
            titulo: "Mi Banco".into(),
            categorias: vec!["cat-finanzas".into()],
            perfil_id: Some("perfil-personal".into()),
            tipo: TipoEntrada::Tarjeta,
            datos_tarjeta: Some(DatosTarjetaBancaria {
                numero: "4532".into(),
                titular: "Titular".into(),
                ..Default::default()
            }),
            ..Default::default()
        };

        let json = serde_json::to_string(&entrada).unwrap();
        // Verificar claves camelCase y compatibilidad con Android
        assert!(json.contains("\"colecciones\":[\"cat-finanzas\"]"));
        assert!(json.contains("\"identidadId\":\"perfil-personal\""));
        assert!(json.contains("\"datosTarjeta\""));

        let des: Entrada = serde_json::from_str(&json).unwrap();
        assert_eq!(entrada, des);
    }
}
