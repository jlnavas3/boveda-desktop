/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Derivación de claves criptográficas utilizando Argon2id v0x13 y limpieza segura de RAM.

use crate::crypto::error::BvdaError;
use argon2::{Algorithm, Argon2, Params, Version};
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

/// Parámetros de resistencia computacional de la función de derivación de claves (KDF).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KdfParams {
    pub memory_kib: u32,
    pub iterations: u32,
    pub parallelism: u32,
    pub hash_length: u32,
}

impl Default for KdfParams {
    fn default() -> Self {
        PerfilArgon2::Estandar.a_kdf_params()
    }
}

/// Perfiles de seguridad Argon2id calibrados en Bóveda Local.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PerfilArgon2 {
    Estandar,
    Reforzado,
    Ultraseguro,
}

impl PerfilArgon2 {
    pub const fn clave(&self) -> &'static str {
        match self {
            Self::Estandar => "estandar",
            Self::Reforzado => "reforzado",
            Self::Ultraseguro => "ultraseguro",
        }
    }

    pub const fn titulo(&self) -> &'static str {
        match self {
            Self::Estandar => "Estándar (64 MiB)",
            Self::Reforzado => "Reforzado (128 MiB)",
            Self::Ultraseguro => "Ultra-Seguro (256 MiB)",
        }
    }

    pub const fn memory_kib(&self) -> u32 {
        match self {
            Self::Estandar => 65_536,
            Self::Reforzado => 131_072,
            Self::Ultraseguro => 262_144,
        }
    }

    pub const fn iterations(&self) -> u32 {
        match self {
            Self::Estandar => 3,
            Self::Reforzado => 4,
            Self::Ultraseguro => 6,
        }
    }

    pub const fn parallelism(&self) -> u32 {
        4
    }

    pub fn a_kdf_params(&self) -> KdfParams {
        KdfParams {
            memory_kib: self.memory_kib(),
            iterations: self.iterations(),
            parallelism: self.parallelism(),
            hash_length: 32,
        }
    }

    pub fn desde_clave(clave: &str) -> Self {
        match clave.to_lowercase().as_str() {
            "reforzado" => Self::Reforzado,
            "ultraseguro" => Self::Ultraseguro,
            _ => Self::Estandar,
        }
    }
}

/// Deriva una clave simétrica de 32 bytes (256 bits) usando Argon2id v0x13.
/// La clave resultante se envuelve en `Zeroizing` para ser borrada de la memoria RAM al destruirse.
pub fn derivar_clave(
    password: &[u8],
    salt: &[u8],
    params: &KdfParams,
) -> Result<Zeroizing<[u8; 32]>, BvdaError> {
    let argon_params = Params::new(
        params.memory_kib,
        params.iterations,
        params.parallelism,
        Some(params.hash_length as usize),
    )
    .map_err(|_| BvdaError::ParametrosArgon2Invalidos)?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon_params);
    let mut clave = Zeroizing::new([0u8; 32]);

    argon2
        .hash_password_into(password, salt, &mut *clave)
        .map_err(|_| BvdaError::ParametrosArgon2Invalidos)?;

    Ok(clave)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perfiles_argon2() {
        assert_eq!(PerfilArgon2::Estandar.memory_kib(), 65_536);
        assert_eq!(PerfilArgon2::Estandar.iterations(), 3);
        assert_eq!(PerfilArgon2::Reforzado.memory_kib(), 131_072);
        assert_eq!(PerfilArgon2::Ultraseguro.memory_kib(), 262_144);

        assert_eq!(PerfilArgon2::desde_clave("reforzado"), PerfilArgon2::Reforzado);
        assert_eq!(PerfilArgon2::desde_clave("desconocido"), PerfilArgon2::Estandar);
    }

    #[test]
    fn test_derivar_clave_determinista() {
        let salt = [0x11u8; 16];
        let params = KdfParams {
            memory_kib: 1024, // Usar mínimo para prueba unitaria ultra rápida
            iterations: 1,
            parallelism: 1,
            hash_length: 32,
        };

        let clave1 = derivar_clave(b"prueba-secreta", &salt, &params).expect("derivar 1");
        let clave2 = derivar_clave(b"prueba-secreta", &salt, &params).expect("derivar 2");
        let clave_distinta = derivar_clave(b"otra-clave", &salt, &params).expect("derivar 3");

        assert_eq!(*clave1, *clave2);
        assert_ne!(*clave1, *clave_distinta);
    }
}
