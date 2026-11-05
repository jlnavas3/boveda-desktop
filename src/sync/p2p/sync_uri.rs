/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

use crate::sync::p2p::fingerprint::calcular_huella_visual;
use crate::sync::p2p::rfc1918::es_ip_privada_rfc1918;

pub const ESQUEMA_SYNC: &str = "bvda-sync://";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncQrPayload {
    pub ips: Vec<String>,
    pub puerto: u16,
    pub clave_publica: [u8; 32],
    pub token_psk: [u8; 32],
    pub version: u32,
    pub alias_dispositivo: Option<String>,
}

impl SyncQrPayload {
    /// Crea un nuevo payload QR de sincronización con la versión canónica actual (2).
    pub fn new(
        ips: Vec<String>,
        puerto: u16,
        clave_publica: [u8; 32],
        token_psk: [u8; 32],
    ) -> Self {
        Self {
            ips,
            puerto,
            clave_publica,
            token_psk,
            version: 2,
            alias_dispositivo: None,
        }
    }

    /// Asigna el alias legible del dispositivo emisor.
    pub fn with_alias(mut self, alias: impl Into<String>) -> Self {
        self.alias_dispositivo = Some(alias.into());
        self
    }

    /// Huella visual determinista de 6 bytes en mayúsculas (ej. "74:5A:63:02:1A:A5").
    pub fn huella_visual(&self) -> String {
        calcular_huella_visual(&self.clave_publica, &self.token_psk)
    }

    /// Clave pública codificada en 64 caracteres hexadecimales minúsculas.
    pub fn clave_publica_hex(&self) -> String {
        bytes_a_hex(&self.clave_publica)
    }

    /// Token PSK codificado en 64 caracteres hexadecimales minúsculas.
    pub fn token_psk_hex(&self) -> String {
        bytes_a_hex(&self.token_psk)
    }
}

pub fn bytes_a_hex(bytes: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(s, "{:02x}", b);
    }
    s
}

pub fn hex_a_bytes32(hex: &str) -> Result<[u8; 32], String> {
    if hex.len() != 64 {
        return Err(format!(
            "Se esperaban 64 caracteres hex, se obtuvieron {}",
            hex.len()
        ));
    }
    let mut bytes = [0u8; 32];
    for i in 0..32 {
        bytes[i] = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)
            .map_err(|e| format!("Carácter hexadecimal inválido en posición {}: {e}", i * 2))?;
    }
    Ok(bytes)
}

/// Comprueba de forma rápida si una cadena inicia con el esquema `bvda-sync://`.
pub fn es_uri_sync(texto: &str) -> bool {
    texto.trim().to_lowercase().starts_with(ESQUEMA_SYNC)
}

/// Parsea un URI `bvda-sync://<ips>:<puerto>?k=<k_hex>&t=<t_hex>&v=<version>`.
pub fn parsear_sync_uri(texto: &str) -> Result<SyncQrPayload, String> {
    let limpio = texto.trim();
    if !limpio.to_lowercase().starts_with(ESQUEMA_SYNC) {
        return Err(format!("Esquema inválido. Debe comenzar con '{ESQUEMA_SYNC}'"));
    }

    let sin_esquema = &limpio[ESQUEMA_SYNC.len()..];
    let mut partes = sin_esquema.splitn(2, '?');
    let autoridad = partes.next().unwrap_or("").trim();
    let query = partes
        .next()
        .ok_or_else(|| "URI malformada: falta el separador de parámetros '?'".to_string())?
        .trim();

    // 1. Autoridad: ips:puerto
    let colon_idx = autoridad.rfind(':').ok_or_else(|| {
        "Autoridad malformada: debe contener al menos una IP y un puerto separado por ':'".to_string()
    })?;

    if colon_idx == 0 || colon_idx >= autoridad.len() - 1 {
        return Err("Autoridad malformada: debe contener IP antes de ':' y puerto después".to_string());
    }

    let hosts_str = &autoridad[..colon_idx];
    let puerto_str = &autoridad[colon_idx + 1..];

    let puerto = puerto_str
        .parse::<u16>()
        .map_err(|_| format!("Puerto inválido: '{puerto_str}'"))?;

    if !(1024..=65535).contains(&puerto) {
        return Err(format!(
            "Puerto fuera del rango permitido (1024..65535): {puerto}"
        ));
    }

    let ips: Vec<String> = hosts_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if ips.is_empty() {
        return Err("Debe especificarse al menos una dirección IP.".to_string());
    }

    for ip in &ips {
        if !es_ip_privada_rfc1918(ip) {
            return Err(format!(
                "Dirección IP no permitida (debe ser IPv4 privada RFC 1918): '{ip}'"
            ));
        }
    }

    // 2. Query params (k, t, v, d)
    let mut k_hex: Option<&str> = None;
    let mut t_hex: Option<&str> = None;
    let mut version: Option<u32> = None;
    let mut alias_dispositivo: Option<String> = None;

    for param in query.split('&') {
        if let Some((k, v)) = param.split_once('=') {
            match k.trim().to_lowercase().as_str() {
                "k" => k_hex = Some(v.trim()),
                "t" => t_hex = Some(v.trim()),
                "v" => version = v.trim().parse::<u32>().ok(),
                "d" => {
                    let decoded = urlencoding::decode(v.trim())
                        .map(|c| c.into_owned())
                        .unwrap_or_else(|_| v.trim().to_string());
                    alias_dispositivo = Some(decoded);
                }
                _ => {}
            }
        }
    }

    let k_str = k_hex.ok_or_else(|| "Parámetro obligatorio 'k' (clave pública) ausente.".to_string())?;
    let t_str = t_hex.ok_or_else(|| "Parámetro obligatorio 't' (token PSK) ausente.".to_string())?;

    let v_num = version.ok_or_else(|| {
        "Versión de protocolo no soportada o ausente (se requiere v>=2)".to_string()
    })?;
    if v_num < 2 {
        return Err(format!(
            "Versión de protocolo no soportada o ausente (se requiere v>=2): '{v_num}'"
        ));
    }

    let clave_publica = hex_a_bytes32(k_str).map_err(|e| format!("Parámetro 'k' inválido: {e}"))?;
    let token_psk = hex_a_bytes32(t_str).map_err(|e| format!("Parámetro 't' inválido: {e}"))?;

    Ok(SyncQrPayload {
        ips,
        puerto,
        clave_publica,
        token_psk,
        version: v_num,
        alias_dispositivo,
    })
}

/// Serializa un [SyncQrPayload] al formato canónico de URI QR:
/// `bvda-sync://${ips.join(",")}:${puerto}?k=${k_hex}&t=${t_hex}&v=${version}&d=${alias}`
pub fn serializar_sync_uri(payload: &SyncQrPayload) -> String {
    let ips_str = payload.ips.join(",");
    let base = format!(
        "{}{}:{puerto}?k={}&t={}&v={}",
        ESQUEMA_SYNC,
        ips_str,
        payload.clave_publica_hex(),
        payload.token_psk_hex(),
        payload.version,
        puerto = payload.puerto,
    );
    if let Some(ref alias) = payload.alias_dispositivo {
        if !alias.trim().is_empty() {
            let encoded = urlencoding::encode(alias.trim());
            return format!("{base}&d={encoded}");
        }
    }
    base
}
