/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Submódulo de comunicación, sincronización y emparejamiento LAN P2P.

pub mod device_name;
pub mod fingerprint;
pub mod handshake;
pub mod lan_ip_detector;
pub mod optical_qr;
pub mod rfc1918;
pub mod session_crypto;
pub mod sync_uri;
pub mod tcp_client;
pub mod tcp_server;
pub mod wire_protocol;

pub use device_name::{formatear_nombre_desktop, obtener_nombre_desktop, SaludoP2P};
pub use fingerprint::calcular_huella_visual;
pub use handshake::{
    calcular_diffie_hellman, construir_client_hello, derivar_clave_publica_x25519,
    derivar_claves_sesion, responder_server_hello, verificar_server_hello, HandshakeKeys,
};
pub use lan_ip_detector::detectar_ips_lan_candidatas;
pub use optical_qr::{
    calcular_crc32_hex, desempaquetar, empaquetar, es_uri_optica, extraer_delta_reciente,
    EnsambladorRafagaOptica, PaqueteOpticoGenerado, TramaOpticaQr, ESQUEMA_OPTICO,
    LIMITE_DELTA_HORAS_DEFECTO, MAX_BYTES_POR_CHUNK_DEFECTO, MAX_CHUNK_BYTES,
};
pub use rfc1918::es_ip_privada_rfc1918;
pub use session_crypto::SessionCipher;
pub use sync_uri::{
    es_uri_sync, parsear_sync_uri, serializar_sync_uri, SyncQrPayload, ESQUEMA_SYNC,
};
pub use tcp_client::{ejecutar_cliente_sync_p2p, ConfiguracionClienteP2P};
pub use tcp_server::{ConfiguracionServidorP2P, ResultadoSyncServidor, ServidorSyncP2P};
pub use wire_protocol::{
    deserializar_mensaje, escribir_trama, leer_trama, serializar_mensaje, MensajeP2P,
    MAX_TRAMA_TAMANO_DEFECTO, TIPO_MSG_COMMIT_ACK, TIPO_MSG_SYNC_PAYLOAD,
};
