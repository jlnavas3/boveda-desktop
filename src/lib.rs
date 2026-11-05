/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Bóveda Local Core - Biblioteca criptográfica y de dominio en Rust.
//! Motor de alta seguridad para Bóveda Local Desktop (Linux y Windows)
//! con compatibilidad binaria estricta con Android.

pub mod config;
pub mod crypto;
pub mod gui;
pub mod models;
pub mod sync;

pub use config::AppConfigDesktop;
pub use crypto::{
    calcular_fuerza_contrasena, cifrar_aes_gcm, comprimir, derivar_clave, descifrar_aes_gcm,
    descomprimir, desempaquetar_boveda, empaquetar_boveda, escribir_atomico, generar_contrasena,
    BvdaError, CabeceraBvda, KdfParams, PasswordGeneratorConfig, PerfilArgon2, MAGIC_BLZ1,
    MAGIC_BVDA, TAM_CABECERA,
};
pub use models::{
    AdjuntoBoveda, CambioContrasena, CampoPersonalizado, Categoria, ConfiguracionBovedaExportable,
    ContenidoBoveda, DatosContacto, DatosCuentaBancaria, DatosCriptoWallet,
    DatosDocumentoIdentidad, DatosPasskey, DatosRedWifi, DatosServidorSsh, DatosTarjetaBancaria,
    Entrada, Perfil, TipoCampo, TipoEntrada, TotpSecundario,
};
pub use sync::{
    calcular_content_hash, calcular_crc32_hex, calcular_diff, calcular_diffie_hellman,
    calcular_huella_visual, consolidar_passkey, construir_client_hello, de_entrada_nueva,
    derivar_clave_publica_x25519, derivar_claves_sesion, desempaquetar, deserializar_mensaje,
    detectar_delimitador, detectar_ips_lan_candidatas, ejecutar_cliente_sync_p2p, empaquetar,
    es_ip_privada_rfc1918, es_uri_optica, es_uri_sync, es_url_web, escribir_trama,
    evaluar_estado_entrada, exportar_csv, exportar_json_plano, extraer_delta_reciente,
    extraer_paquete_app, formatear_nombre_desktop, fusionar_bovedas, fusionar_bovedas_con_informe,
    fusionar_entradas, fusionar_submodelos, importar_bitwarden_json, importar_csv, leer_trama,
    obtener_nombre_desktop, parsear_sync_uri, responder_server_hello, serializar_mensaje, serializar_sync_uri, son_misma_passkey,
    verificar_server_hello, ConfiguracionClienteP2P, ConfiguracionServidorP2P, DecisionArbitro,
    EnsambladorRafagaOptica, HandshakeKeys, InformeSincronizacion, ItemCambioSync, MascaraCampos,
    MensajeP2P, PaqueteOpticoGenerado, ResultadoFusionNativa, ResultadoSyncServidor,
    ServidorSyncP2P, SessionCipher, SyncQrPayload, TramaOpticaQr, TipoCambioSync, ESQUEMA_OPTICO,
    ESQUEMA_SYNC, LIMITE_DELTA_HORAS_DEFECTO, MAX_BYTES_POR_CHUNK_DEFECTO, MAX_CHUNK_BYTES,
    MAX_TRAMA_TAMANO_DEFECTO, TIPO_MSG_COMMIT_ACK, TIPO_MSG_SYNC_PAYLOAD,
};
