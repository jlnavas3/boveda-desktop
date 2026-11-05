/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Códec criptográfico, fragmentador y ensamblador para el canal de transporte óptico por QR (`bvda-opt://`).
//! Diseñado para sincronización unidireccional con dispositivos air-gapped o sin permisos de red local.
//!
//! Protocolo:
//! - Compresión Zlib estándar (RFC 1950) con compresión óptima.
//! - Cifrado autenticado ChaCha20-Poly1305 protegido por PIN efímero de 6 dígitos y sesión hex de 4 bytes.
//! - Integridad CRC32 IEEE estándar calculada sobre el bloque cifrado total.
//! - Fragmentación en tramas URL-safe Base64 sin padding (máximo 600 bytes por defecto).
//! - Reensamblado tolerante a ráfagas animadas fuera de orden o con tramas duplicadas.

use std::collections::{BTreeMap, HashSet};
use std::io::{Read, Write};

use base64::prelude::*;
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    ChaCha20Poly1305, Key, Nonce,
};
use crc32fast::Hasher as Crc32Hasher;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use rand::Rng;
use sha2::{Digest, Sha256};

use crate::models::contenido::ContenidoBoveda;
use crate::models::entrada::Entrada;

pub const ESQUEMA_OPTICO: &str = "bvda-opt://";
pub const MAX_BYTES_POR_CHUNK_DEFECTO: usize = 300;
pub const MAX_CHUNK_BYTES: usize = MAX_BYTES_POR_CHUNK_DEFECTO;
pub const LIMITE_DELTA_HORAS_DEFECTO: i64 = 24;

/// Paquete empaquetado para transmisión óptica mediante ráfaga o QR estático.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaqueteOpticoGenerado {
    pub id_sesion_hex: String,
    pub pin_efimero: String,
    pub total_tramas: usize,
    pub tramas: Vec<TramaOpticaQr>,
    pub bytes_cifrados_totales: usize,
    pub es_delta: bool,
}

/// Representa una trama individual de código QR óptico (`bvda-opt://`).
///
/// Formato canónico:
/// `bvda-opt://v<version>/<indice>/<total>/<id_sesion_hex8>/<crc32_total_hex8>?d=<chunk_base64url>`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TramaOpticaQr {
    pub version: u32,
    pub indice: usize,
    pub total: usize,
    pub id_sesion_hex: String,
    pub crc32_total_hex: String,
    pub datos_chunk: Vec<u8>,
    pub alias_emisor: Option<String>,
}

impl TramaOpticaQr {
    /// Serializa la trama al formato canónico de URI óptico.
    pub fn serializar_uri(&self) -> String {
        let b64 = BASE64_URL_SAFE_NO_PAD.encode(&self.datos_chunk);
        let id_norm = self.id_sesion_hex.to_uppercase();
        let crc_norm = self.crc32_total_hex.to_uppercase();
        let alias_param = match &self.alias_emisor {
            Some(a) if !a.trim().is_empty() => format!("&a={}", urlencoding::encode(a.trim())),
            _ => String::new(),
        };
        format!(
            "{ESQUEMA_OPTICO}v{}/{}/{}/{id_norm}/{crc_norm}?d={b64}{alias_param}",
            self.version, self.indice, self.total
        )
    }

    /// Determina si una cadena contiene el prefijo del esquema óptico.
    pub fn es_uri_optica(uri: &str) -> bool {
        es_uri_optica(uri)
    }

    /// Parsea un URI óptico al modelo estructurado `TramaOpticaQr`.
    pub fn parsear_uri(uri: &str) -> Option<Self> {
        let u = uri.trim();
        if !es_uri_optica(u) {
            return None;
        }

        let sin_esquema = &u[ESQUEMA_OPTICO.len()..];
        let (ruta, query) = sin_esquema.split_once('?')?;

        let partes_ruta: Vec<&str> = ruta.split('/').collect();
        if partes_ruta.len() < 5 {
            return None;
        }

        let ver_str = partes_ruta[0]
            .strip_prefix('v')
            .or_else(|| partes_ruta[0].strip_prefix('V'))
            .unwrap_or(partes_ruta[0]);
        let version: u32 = ver_str.parse().unwrap_or(1);

        let indice: usize = partes_ruta[1].parse().ok()?;
        let total: usize = partes_ruta[2].parse().ok()?;
        let id_sesion = partes_ruta[3].to_uppercase();
        let crc32 = partes_ruta[4].to_uppercase();

        if indice < 1 || total < 1 || indice > total {
            return None;
        }
        if id_sesion.len() != 8 || crc32.len() != 8 {
            return None;
        }

        let mut payload_b64: Option<&str> = None;
        let mut alias_emisor: Option<String> = None;
        for param in query.split('&') {
            if let Some((k, v)) = param.split_once('=') {
                match k {
                    "d" => payload_b64 = Some(v),
                    "a" => {
                        let decoded = urlencoding::decode(v)
                            .map(|c| c.into_owned())
                            .unwrap_or_else(|_| v.to_string());
                        alias_emisor = Some(decoded);
                    }
                    _ => {}
                }
            }
        }

        let payload = payload_b64?;
        if payload.is_empty() {
            return None;
        }

        let datos_chunk = BASE64_URL_SAFE_NO_PAD.decode(payload).ok()?;

        Some(TramaOpticaQr {
            version,
            indice,
            total,
            id_sesion_hex: id_sesion,
            crc32_total_hex: crc32,
            datos_chunk,
            alias_emisor,
        })
    }
}

/// Determina si una cadena es un URI del esquema `bvda-opt://`.
pub fn es_uri_optica(uri: &str) -> bool {
    let trimmed = uri.trim();
    if trimmed.len() < ESQUEMA_OPTICO.len() {
        return false;
    }
    trimmed[..ESQUEMA_OPTICO.len()].eq_ignore_ascii_case(ESQUEMA_OPTICO)
}

/// Calcula el CRC32 IEEE estándar formateado en 8 caracteres hexadecimales en mayúsculas.
pub fn calcular_crc32_hex(datos: &[u8]) -> String {
    let mut hasher = Crc32Hasher::new();
    hasher.update(datos);
    format!("{:08X}", hasher.finalize())
}

/// Deriva la clave de 32 bytes para ChaCha20-Poly1305 a partir del ID de sesión y el PIN.
fn derivar_clave_optica(id_sesion_hex: &str, pin_6_digitos: &str) -> [u8; 32] {
    let msg = format!(
        "BVDA-OPT-V1:{}:{}",
        id_sesion_hex.to_uppercase(),
        pin_6_digitos.trim()
    );
    let mut hasher = Sha256::new();
    hasher.update(msg.as_bytes());
    let mut clave = [0u8; 32];
    clave.copy_from_slice(&hasher.finalize());
    clave
}

/// Comprime un búfer de bytes con Zlib estándar (RFC 1950) y máxima compresión.
fn comprimir_zlib(datos: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(datos)?;
    encoder.finish()
}

/// Descomprime un búfer Zlib estándar (RFC 1950).
fn descomprimir_zlib(datos: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut decoder = ZlibDecoder::new(datos);
    let mut buffer = Vec::new();
    decoder.read_to_end(&mut buffer)?;
    Ok(buffer)
}

/// Cifra un bloque de datos con ChaCha20-Poly1305, Nonce de 12 ceros y AAD de sesión.
fn cifrar_optico(
    datos_comprimidos: &[u8],
    id_sesion_hex: &str,
    pin_6_digitos: &str,
) -> Result<Vec<u8>, String> {
    let clave = derivar_clave_optica(id_sesion_hex, pin_6_digitos);
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&clave));
    let nonce = Nonce::from_slice(&[0u8; 12]);
    let aad = format!("BVDA-OPT-V1:{}", id_sesion_hex.to_uppercase());
    let payload = Payload {
        msg: datos_comprimidos,
        aad: aad.as_bytes(),
    };
    cipher
        .encrypt(nonce, payload)
        .map_err(|e| format!("Error al cifrar bloque óptico: {e}"))
}

/// Descifra un bloque de datos con ChaCha20-Poly1305 autenticado.
fn descifrar_optico(
    bloque_cifrado: &[u8],
    id_sesion_hex: &str,
    pin_6_digitos: &str,
) -> Result<Vec<u8>, String> {
    let clave = derivar_clave_optica(id_sesion_hex, pin_6_digitos);
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&clave));
    let nonce = Nonce::from_slice(&[0u8; 12]);
    let aad = format!("BVDA-OPT-V1:{}", id_sesion_hex.to_uppercase());
    let payload = Payload {
        msg: bloque_cifrado,
        aad: aad.as_bytes(),
    };
    cipher
        .decrypt(nonce, payload)
        .map_err(|_| "PIN óptico de 6 dígitos incorrecto o código QR alterado.".to_string())
}

/// Extrae las entradas modificadas, creadas o eliminadas en las últimas horas para formar un delta mínimo.
pub fn extraer_delta_reciente(boveda: &ContenidoBoveda, horas_atras: i64) -> ContenidoBoveda {
    let umbral_ms = chrono::Utc::now().timestamp_millis() - (horas_atras * 3600 * 1000);

    let entradas_delta: Vec<Entrada> = boveda
        .entradas
        .iter()
        .filter(|e| {
            e.modificada_en >= umbral_ms || e.creada_en >= umbral_ms || e.eliminada_en >= umbral_ms
        })
        .cloned()
        .collect();

    let entradas_finales = if entradas_delta.is_empty() && !boveda.entradas.is_empty() {
        boveda
            .entradas
            .iter()
            .max_by_key(|e| e.modificada_en.max(e.creada_en))
            .cloned()
            .into_iter()
            .collect()
    } else {
        entradas_delta
    };

    let cat_ids: HashSet<String> = entradas_finales
        .iter()
        .flat_map(|e| e.categorias.iter().cloned())
        .collect();

    let categorias_finales = if cat_ids.is_empty() {
        boveda.categorias.clone()
    } else {
        boveda
            .categorias
            .iter()
            .filter(|c| cat_ids.contains(&c.id) || cat_ids.contains(&c.nombre))
            .cloned()
            .collect()
    };

    let mut boveda_delta = boveda.clone();
    boveda_delta.entradas = entradas_finales;
    boveda_delta.categorias = categorias_finales;
    boveda_delta
}

/// Genera un paquete óptico completo a partir de una bóveda.
pub fn empaquetar(
    boveda: &ContenidoBoveda,
    solo_delta: bool,
    max_bytes_por_chunk: usize,
) -> Result<PaqueteOpticoGenerado, String> {
    let mut boveda_a_enviar = if solo_delta {
        extraer_delta_reciente(boveda, LIMITE_DELTA_HORAS_DEFECTO)
    } else {
        boveda.clone()
    };
    boveda_a_enviar.alias_dispositivo = Some(crate::sync::p2p::device_name::obtener_nombre_desktop());

    // 1. Serializar JSON compacto y comprimir con Zlib
    let json_str = serde_json::to_string(&boveda_a_enviar)
        .map_err(|e| format!("Error al serializar bóveda a JSON: {e}"))?;
    let bytes_comprimidos = comprimir_zlib(json_str.as_bytes())
        .map_err(|e| format!("Error al comprimir bóveda con Zlib: {e}"))?;

    // 2. Generar sesión y PIN efímero de 6 dígitos
    let mut rng = rand::thread_rng();
    let id_sesion_bytes: [u8; 4] = rng.gen();
    let id_sesion_hex = format!(
        "{:02X}{:02X}{:02X}{:02X}",
        id_sesion_bytes[0], id_sesion_bytes[1], id_sesion_bytes[2], id_sesion_bytes[3]
    );

    let pin_num: u32 = rng.gen_range(100_000..=999_999);
    let pin_efimero = pin_num.to_string();

    // 3. Cifrar con ChaCha20-Poly1305
    let bloque_cifrado = cifrar_optico(&bytes_comprimidos, &id_sesion_hex, &pin_efimero)?;
    let crc32_hex = calcular_crc32_hex(&bloque_cifrado);

    // 4. Fragmentar en chunks
    let chunk_size = if max_bytes_por_chunk == 0 {
        MAX_BYTES_POR_CHUNK_DEFECTO
    } else {
        max_bytes_por_chunk
    };

    let mut chunks = Vec::new();
    let mut offset = 0;
    while offset < bloque_cifrado.len() {
        let end = (offset + chunk_size).min(bloque_cifrado.len());
        chunks.push(bloque_cifrado[offset..end].to_vec());
        offset = end;
    }

    if chunks.is_empty() {
        chunks.push(Vec::new());
    }

    let total = chunks.len();
    let alias_local = crate::sync::p2p::device_name::obtener_nombre_desktop();
    let tramas = chunks
        .into_iter()
        .enumerate()
        .map(|(idx, chunk)| TramaOpticaQr {
            version: 1,
            indice: idx + 1,
            total,
            id_sesion_hex: id_sesion_hex.clone(),
            crc32_total_hex: crc32_hex.clone(),
            datos_chunk: chunk,
            alias_emisor: Some(alias_local.clone()),
        })
        .collect();

    Ok(PaqueteOpticoGenerado {
        id_sesion_hex,
        pin_efimero,
        total_tramas: total,
        tramas,
        bytes_cifrados_totales: bloque_cifrado.len(),
        es_delta: solo_delta,
    })
}

/// Reensambla, valida CRC32, descifra con el PIN de 6 dígitos y descomprime la bóveda.
pub fn desempaquetar(
    bloque_cifrado_completo: &[u8],
    id_sesion_hex: &str,
    pin_6_digitos: &str,
    crc32_esperado_hex: &str,
) -> Result<ContenidoBoveda, String> {
    // 1. Verificar CRC32 del bloque total
    let crc_calculado = calcular_crc32_hex(bloque_cifrado_completo);
    if !crc_calculado.eq_ignore_ascii_case(crc32_esperado_hex) {
        return Err(format!(
            "Fallo de integridad CRC32: esperado {crc32_esperado_hex}, calculado {crc_calculado}"
        ));
    }

    // 2. Descifrar con ChaCha20-Poly1305
    let bytes_comprimidos =
        descifrar_optico(bloque_cifrado_completo, id_sesion_hex, pin_6_digitos.trim())?;

    // 3. Descomprimir Zlib
    let json_bytes = descomprimir_zlib(&bytes_comprimidos)
        .map_err(|e| format!("Error al descomprimir datos Zlib: {e}"))?;

    // 4. Deserializar ContenidoBoveda
    serde_json::from_slice::<ContenidoBoveda>(&json_bytes)
        .map_err(|e| format!("Error deserializando contenido de bóveda JSON: {e}"))
}

/// Acumulador y reensamblador reactivo para tramas de códigos QR ópticos.
/// Soporta recepción de fragmentos 1..N fuera de orden o con tramas duplicadas.
#[derive(Debug, Default, Clone)]
pub struct EnsambladorRafagaOptica {
    sesion_actual_id: Option<String>,
    total_esperado: usize,
    crc32_esperado_hex: Option<String>,
    alias_emisor: Option<String>,
    tramas_recibidas: BTreeMap<usize, Vec<u8>>,
}

impl EnsambladorRafagaOptica {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn sesion_id(&self) -> Option<&str> {
        self.sesion_actual_id.as_deref()
    }

    pub fn alias_emisor(&self) -> Option<&str> {
        self.alias_emisor.as_deref()
    }

    pub fn progreso(&self) -> usize {
        self.tramas_recibidas.len()
    }

    pub fn total(&self) -> usize {
        self.total_esperado
    }

    pub fn esta_completo(&self) -> bool {
        self.total_esperado > 0 && self.tramas_recibidas.len() == self.total_esperado
    }

    pub fn porcentaje(&self) -> f32 {
        if self.total_esperado > 0 {
            self.tramas_recibidas.len() as f32 / self.total_esperado as f32
        } else {
            0.0
        }
    }

    /// Registra una trama capturada. Retorna `true` si es una nueva trama que incrementa el progreso.
    /// Si la trama pertenece a otra sesión, reinicia automáticamente el estado.
    pub fn registrar_trama(&mut self, trama: &TramaOpticaQr) -> bool {
        let id_norm = trama.id_sesion_hex.to_uppercase();
        if self.sesion_actual_id.as_deref() != Some(&id_norm) {
            self.sesion_actual_id = Some(id_norm);
            self.total_esperado = trama.total;
            self.crc32_esperado_hex = Some(trama.crc32_total_hex.to_uppercase());
            self.alias_emisor = trama.alias_emisor.clone();
            self.tramas_recibidas.clear();
        } else if self.alias_emisor.is_none() && trama.alias_emisor.is_some() {
            self.alias_emisor = trama.alias_emisor.clone();
        }

        if trama.total != self.total_esperado {
            return false;
        }

        self.tramas_recibidas
            .insert(trama.indice, trama.datos_chunk.clone())
            .is_none()
    }

    /// Reconstituye el bloque binario concatenando los fragmentos en orden estricto 1..N.
    pub fn ensamblar_bloque_cifrado(&self) -> Result<Vec<u8>, String> {
        if !self.esta_completo() {
            return Err(format!(
                "Ráfaga incompleta: recibidos {} de {} fragmentos.",
                self.tramas_recibidas.len(),
                self.total_esperado
            ));
        }

        let mut bloque = Vec::new();
        for i in 1..=self.total_esperado {
            let chunk = self.tramas_recibidas.get(&i).ok_or_else(|| {
                format!(
                    "Falta el fragmento {} de la sesión {:?}",
                    i, self.sesion_actual_id
                )
            })?;
            bloque.extend_from_slice(chunk);
        }
        Ok(bloque)
    }

    /// Descifra y descomprime la bóveda una vez que la ráfaga está completa.
    pub fn descifrar_boveda(&self, pin_6_digitos: &str) -> Result<ContenidoBoveda, String> {
        let bloque = self.ensamblar_bloque_cifrado()?;
        let sesion = self
            .sesion_actual_id
            .as_deref()
            .ok_or_else(|| "Sesión óptica no iniciada.".to_string())?;
        let crc = self
            .crc32_esperado_hex
            .as_deref()
            .ok_or_else(|| "CRC32 no disponible.".to_string())?;

        desempaquetar(&bloque, sesion, pin_6_digitos, crc)
    }

    /// Reinicia el acumulador a su estado inicial.
    pub fn reiniciar(&mut self) {
        self.sesion_actual_id = None;
        self.total_esperado = 0;
        self.crc32_esperado_hex = None;
        self.alias_emisor = None;
        self.tramas_recibidas.clear();
    }
}
