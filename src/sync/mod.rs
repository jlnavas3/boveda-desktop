/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Módulo de sincronización, fusión inteligente e interoperabilidad externa.

pub mod export;
pub mod import;
pub mod merge;
pub mod p2p;

pub use export::{exportar_csv, exportar_json_plano};
pub use import::{detectar_delimitador, importar_bitwarden_json, importar_csv};
pub use merge::{
    calcular_content_hash, calcular_diff, consolidar_passkey, de_entrada_nueva, es_url_web,
    evaluar_estado_entrada, extraer_paquete_app, fusionar_bovedas, fusionar_bovedas_con_informe,
    fusionar_entradas, fusionar_submodelos, son_misma_passkey, DecisionArbitro,
    InformeSincronizacion, ItemCambioSync, MascaraCampos, ResultadoFusionNativa, TipoCambioSync,
};
pub use p2p::{
    calcular_crc32_hex, calcular_diffie_hellman, calcular_huella_visual, construir_client_hello,
    derivar_clave_publica_x25519, derivar_claves_sesion, desempaquetar, deserializar_mensaje,
    detectar_ips_lan_candidatas, ejecutar_cliente_sync_p2p, empaquetar, es_ip_privada_rfc1918,
    es_uri_optica, es_uri_sync, escribir_trama, extraer_delta_reciente, formatear_nombre_desktop,
    leer_trama, obtener_nombre_desktop, parsear_sync_uri, responder_server_hello,
    serializar_mensaje, serializar_sync_uri, verificar_server_hello, ConfiguracionClienteP2P, ConfiguracionServidorP2P,
    EnsambladorRafagaOptica, HandshakeKeys, MensajeP2P, PaqueteOpticoGenerado,
    ResultadoSyncServidor, SaludoP2P, ServidorSyncP2P, SessionCipher, SyncQrPayload, TramaOpticaQr,
    ESQUEMA_OPTICO, ESQUEMA_SYNC, LIMITE_DELTA_HORAS_DEFECTO, MAX_BYTES_POR_CHUNK_DEFECTO,
    MAX_CHUNK_BYTES, MAX_TRAMA_TAMANO_DEFECTO, TIPO_MSG_COMMIT_ACK, TIPO_MSG_SYNC_PAYLOAD,
};
