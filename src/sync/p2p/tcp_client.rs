/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Cliente TCP para sincronización LAN P2P.
//! Conecta al servidor de sincronización, ejecuta handshake 1-RTT, transmite bóveda local,
//! recibe bóveda fusionada y confirma mediante Commit ACK.

use std::net::TcpStream;
use std::time::Duration;

use crate::models::ContenidoBoveda;
use crate::sync::p2p::handshake::{construir_client_hello, verificar_server_hello};
use crate::sync::p2p::session_crypto::SessionCipher;
use crate::sync::p2p::wire_protocol::{
    deserializar_mensaje, escribir_trama, leer_trama, serializar_mensaje, MensajeP2P,
    MAX_TRAMA_TAMANO_DEFECTO,
};

/// Configuración para iniciar una conexión de cliente P2P.
#[derive(Clone)]
pub struct ConfiguracionClienteP2P {
    /// Clave privada escalar X25519 del cliente (32 bytes).
    pub sk_cliente: [u8; 32],
    /// Clave pública X25519 del servidor remoto (32 bytes).
    pub pk_servidor: [u8; 32],
    /// Token secreto precompartido del QR (32 bytes).
    pub psk_token: [u8; 32],
    /// Dirección IP del host destino.
    pub host: String,
    /// Puerto TCP del servidor.
    pub puerto: u16,
    /// Timeout para operaciones de red (por defecto 60 segundos).
    pub timeout_segundos: u64,
}

impl ConfiguracionClienteP2P {
    pub fn new(
        sk_cliente: [u8; 32],
        pk_servidor: [u8; 32],
        psk_token: [u8; 32],
        host: String,
        puerto: u16,
    ) -> Self {
        Self {
            sk_cliente,
            pk_servidor,
            psk_token,
            host,
            puerto,
            timeout_segundos: 60,
        }
    }
}

/// Ejecuta el ciclo de sincronización como cliente:
/// 1. Conecta vía TCP al host y puerto destino.
/// 2. Envía `ClientHello` (64 bytes).
/// 3. Recibe y valida `ServerHello` (32 bytes).
/// 4. Cifra y envía la bóveda local mediante `SessionCipher`.
/// 5. Recibe y descifra la bóveda fusionada devuelta por el servidor.
/// 6. Envía `CommitAck("OK")` cifrado.
/// 7. Retorna la bóveda fusionada.
pub fn ejecutar_cliente_sync_p2p(
    config: &ConfiguracionClienteP2P,
    boveda_local: &ContenidoBoveda,
) -> Result<ContenidoBoveda, String> {
    let direccion = format!("{}:{}", config.host, config.puerto);
    let timeout = Duration::from_secs(config.timeout_segundos);

    let mut stream = TcpStream::connect(&direccion)
        .map_err(|e| format!("Fallo al conectar cliente TCP con {direccion}: {e}"))?;

    stream
        .set_read_timeout(Some(timeout))
        .map_err(|e| format!("Error configurando timeout de lectura: {e}"))?;
    stream
        .set_write_timeout(Some(timeout))
        .map_err(|e| format!("Error configurando timeout de escritura: {e}"))?;

    // 1. Construir y enviar ClientHello (64 bytes)
    let (client_hello, keys) = construir_client_hello(
        &config.sk_cliente,
        &config.pk_servidor,
        &config.psk_token,
    )?;
    escribir_trama(&mut stream, &client_hello)?;

    // 2. Leer y verificar ServerHello (32 bytes)
    let server_hello_raw = leer_trama(&mut stream, 1024)?;
    if server_hello_raw.len() != 32 {
        return Err(format!(
            "Trama ServerHello inválida: longitud esperada 32 bytes, recibida {}",
            server_hello_raw.len()
        ));
    }
    let mut server_hello = [0u8; 32];
    server_hello.copy_from_slice(&server_hello_raw);

    let mut pk_cliente = [0u8; 32];
    pk_cliente.copy_from_slice(&client_hello[0..32]);

    verificar_server_hello(
        &keys.handshake_key,
        &config.pk_servidor,
        &pk_cliente,
        &server_hello,
    )?;

    // 3. Instanciar cifrador de sesión como cliente
    let mut cipher = SessionCipher::para_cliente(&keys);

    // 4. Cifrar y enviar la bóveda local con la identidad del equipo si no fue provista
    let mut boveda_a_enviar = boveda_local.clone();
    if boveda_a_enviar.alias_dispositivo.is_none() {
        boveda_a_enviar.alias_dispositivo = Some(crate::sync::p2p::device_name::obtener_nombre_desktop());
    }
    let msg_local = MensajeP2P::SyncPayload(Box::new(boveda_a_enviar));
    let plaintext_local = serializar_mensaje(&msg_local)?;
    let frame_local = cipher.seal(&plaintext_local)?;
    escribir_trama(&mut stream, &frame_local)?;

    // 5. Recibir y descifrar la bóveda fusionada
    let frame_fusionada = leer_trama(&mut stream, MAX_TRAMA_TAMANO_DEFECTO)?;
    let plaintext_fusionada = cipher.open(&frame_fusionada)?;
    let msg_fusionada = deserializar_mensaje(&plaintext_fusionada)?;

    let boveda_fusionada = match msg_fusionada {
        MensajeP2P::SyncPayload(b) => *b,
        MensajeP2P::CommitAck(_) => {
            return Err("Protocolo violado: se esperaba SyncPayload del servidor".into());
        }
    };

    // 6. Enviar confirmación CommitAck
    let msg_ack = MensajeP2P::CommitAck("OK".to_string());
    let plaintext_ack = serializar_mensaje(&msg_ack)?;
    let frame_ack = cipher.seal(&plaintext_ack)?;
    escribir_trama(&mut stream, &frame_ack)?;

    // 7. Cierre limpio
    let _ = stream.shutdown(std::net::Shutdown::Both);

    Ok(boveda_fusionada)
}
