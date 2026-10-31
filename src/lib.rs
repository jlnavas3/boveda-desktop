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
    consolidar_passkey, detectar_delimitador, exportar_csv, exportar_json_plano,
    fusionar_bovedas, fusionar_entradas, fusionar_submodelos, importar_bitwarden_json,
    importar_csv, son_misma_passkey, ResultadoFusionNativa,
};
