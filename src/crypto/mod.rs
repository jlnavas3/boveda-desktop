/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Módulo criptográfico y de manipulación de contenedores .bvda.

pub mod cipher;
pub mod common_passwords;
pub mod compressor;
pub mod error;
pub mod generator;
pub mod header;
pub mod kdf;
pub mod strength_meter;
pub mod totp;
pub mod vault;

pub use cipher::{cifrar_aes_gcm, descifrar_aes_gcm};
pub use common_passwords::es_contrasena_comun;
pub use compressor::{comprimir, descomprimir, es_comprimido, MAGIC_BLZ1};
pub use error::BvdaError;
pub use generator::{calcular_fuerza_contrasena, generar_contrasena, PasswordGeneratorConfig};
pub use header::{
    CabeceraBvda, MAGIC_BVDA, TAM_CABECERA, TAM_NONCE, TAM_SALT, TAM_TAG_GCM, VERSION_BVDA,
};
pub use kdf::{derivar_clave, KdfParams, PerfilArgon2};
pub use strength_meter::{medir_fuerza_contrasena, FuerzaContrasena};
pub use totp::{
    codigo_totp, codigo_totp_desde_base32, decodificar_base32, formatear_codigo_totp,
    segundos_restantes_totp,
};
pub use vault::{desempaquetar_boveda, empaquetar_boveda, escribir_atomico};
