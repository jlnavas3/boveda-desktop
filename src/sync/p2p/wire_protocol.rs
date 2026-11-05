/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Protocolo de tramas de red (Framing) y serialización de mensajes P2P.
//! Contrato: Prefijo de longitud u32 little-endian [len_le (4B)] || [payload (len bytes)].

use std::io::{Read, Write};

use crate::models::ContenidoBoveda;

/// Límite de seguridad para evitar desbordamientos de memoria (OOM): 64 MB.
pub const MAX_TRAMA_TAMANO_DEFECTO: usize = 64 * 1024 * 1024;

/// Identificador de tipo de mensaje: Bóveda cifrada completa.
pub const TIPO_MSG_SYNC_PAYLOAD: u8 = 0x01;

/// Identificador de tipo de mensaje: Confirmación de Commit exitoso.
pub const TIPO_MSG_COMMIT_ACK: u8 = 0x02;

/// Mensajes de alto nivel intercambiados a través del túnel seguro P2P.
#[derive(Debug, Clone, PartialEq)]
pub enum MensajeP2P {
    /// Bóveda de contraseñas serializada para sincronización bidireccional.
    SyncPayload(Box<ContenidoBoveda>),
    /// Confirmación de recepción y commit persistido.
    CommitAck(String),
}

/// Lee una trama con prefijo de longitud `u32` little-endian desde un flujo de lectura.
pub fn leer_trama<R: Read>(reader: &mut R, max_bytes: usize) -> Result<Vec<u8>, String> {
    let mut len_buf = [0u8; 4];
    reader
        .read_exact(&mut len_buf)
        .map_err(|e| format!("Error leyendo prefijo de longitud de trama: {e}"))?;

    let len = u32::from_le_bytes(len_buf) as usize;
    if len > max_bytes {
        return Err(format!(
            "Trama rechazada por exceder tamaño máximo permitido: {len} bytes (máximo {max_bytes})"
        ));
    }

    let mut payload = vec![0u8; len];
    reader
        .read_exact(&mut payload)
        .map_err(|e| format!("Error leyendo cuerpo de trama ({len} bytes): {e}"))?;

    Ok(payload)
}

/// Escribe una trama con prefijo de longitud `u32` little-endian en un flujo de escritura.
pub fn escribir_trama<W: Write>(writer: &mut W, payload: &[u8]) -> Result<(), String> {
    let len = payload.len();
    if len > u32::MAX as usize {
        return Err(format!("Trama demasiado grande para u32: {len} bytes"));
    }

    let len_buf = (len as u32).to_le_bytes();
    writer
        .write_all(&len_buf)
        .map_err(|e| format!("Error escribiendo prefijo de longitud de trama: {e}"))?;

    writer
        .write_all(payload)
        .map_err(|e| format!("Error escribiendo cuerpo de trama: {e}"))?;

    writer
        .flush()
        .map_err(|e| format!("Error vaciando buffer de red: {e}"))?;

    Ok(())
}

/// Serializa un mensaje P2P al formato binario canónico: `[tipo (1B)] || [cuerpo]`.
pub fn serializar_mensaje(msg: &MensajeP2P) -> Result<Vec<u8>, String> {
    match msg {
        MensajeP2P::SyncPayload(boveda) => {
            let json_bytes = serde_json::to_vec(boveda)
                .map_err(|e| format!("Error serializando ContenidoBoveda a JSON: {e}"))?;
            let mut out = Vec::with_capacity(1 + json_bytes.len());
            out.push(TIPO_MSG_SYNC_PAYLOAD);
            out.extend_from_slice(&json_bytes);
            Ok(out)
        }
        MensajeP2P::CommitAck(ack) => {
            let ack_bytes = ack.as_bytes();
            let mut out = Vec::with_capacity(1 + ack_bytes.len());
            out.push(TIPO_MSG_COMMIT_ACK);
            out.extend_from_slice(ack_bytes);
            Ok(out)
        }
    }
}

/// Deserializa un mensaje P2P desde el formato binario canónico.
pub fn deserializar_mensaje(bytes: &[u8]) -> Result<MensajeP2P, String> {
    if bytes.is_empty() {
        return Err("Mensaje P2P vacío".to_string());
    }

    let tipo = bytes[0];
    let cuerpo = &bytes[1..];

    match tipo {
        TIPO_MSG_SYNC_PAYLOAD => {
            let boveda: ContenidoBoveda = serde_json::from_slice(cuerpo)
                .map_err(|e| format!("Error deserializando ContenidoBoveda desde JSON: {e}"))?;
            Ok(MensajeP2P::SyncPayload(Box::new(boveda)))
        }
        TIPO_MSG_COMMIT_ACK => {
            let ack = String::from_utf8(cuerpo.to_vec())
                .map_err(|e| format!("ACK contiene UTF-8 inválido: {e}"))?;
            Ok(MensajeP2P::CommitAck(ack))
        }
        otro => Err(format!("Tipo de mensaje P2P desconocido: 0x{:02x}", otro)),
    }
}
