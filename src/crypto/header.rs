/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Parseo, validación y serialización de la cabecera inmutable de 49 bytes del archivo .bvda.

use crate::crypto::error::BvdaError;
use byteorder::{BigEndian, ByteOrder};

pub const MAGIC_BVDA: &[u8; 4] = b"BVDA";
pub const VERSION_BVDA: u8 = 1;
pub const TAM_SALT: usize = 16;
pub const TAM_NONCE: usize = 12;
pub const TAM_TAG_GCM: usize = 16;
pub const TAM_CABECERA: usize = 4 + 1 + TAM_SALT + 16 + TAM_NONCE; // 49 Bytes

/// Representa la cabecera binaria inmutable de 49 bytes del archivo `.bvda`.
/// Coincide al 100% con VaultCrypto.Cabecera en la app Android.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CabeceraBvda {
    pub version: u8,
    pub salt: [u8; TAM_SALT],
    pub memory_kib: u32,
    pub iterations: u32,
    pub parallelism: u32,
    pub hash_length: u32,
    pub nonce: [u8; TAM_NONCE],
}

impl CabeceraBvda {
    /// Crea una nueva cabecera validando los parámetros de Argon2id.
    pub fn nueva(
        salt: [u8; TAM_SALT],
        memory_kib: u32,
        iterations: u32,
        parallelism: u32,
        nonce: [u8; TAM_NONCE],
    ) -> Result<Self, BvdaError> {
        let hash_length = 32;
        if !(1_024..=1_048_576).contains(&memory_kib)
            || !(1..=64).contains(&iterations)
            || !(1..=64).contains(&parallelism)
        {
            return Err(BvdaError::ParametrosArgon2Invalidos);
        }

        Ok(Self {
            version: VERSION_BVDA,
            salt,
            memory_kib,
            iterations,
            parallelism,
            hash_length,
            nonce,
        })
    }

    /// Parsea y valida los primeros 49 bytes de un archivo `.bvda`.
    pub fn parsear(bytes: &[u8]) -> Result<Self, BvdaError> {
        if bytes.len() < TAM_CABECERA + TAM_TAG_GCM {
            return Err(BvdaError::ArchivoDemasiadoCorto);
        }

        if &bytes[0..4] != MAGIC_BVDA {
            return Err(BvdaError::MagicoInvalido);
        }

        let version = bytes[4];
        if version != VERSION_BVDA {
            return Err(BvdaError::VersionNoSoportada(version));
        }

        let mut salt = [0u8; TAM_SALT];
        salt.copy_from_slice(&bytes[5..21]);

        let memory_kib = BigEndian::read_u32(&bytes[21..25]);
        let iterations = BigEndian::read_u32(&bytes[25..29]);
        let parallelism = BigEndian::read_u32(&bytes[29..33]);
        let hash_length = BigEndian::read_u32(&bytes[33..37]);

        // Validación idéntica a VaultCrypto.kt en Android
        if !(1_024..=1_048_576).contains(&memory_kib)
            || !(1..=64).contains(&iterations)
            || !(1..=64).contains(&parallelism)
            || hash_length != 32
        {
            return Err(BvdaError::ParametrosArgon2Invalidos);
        }

        let mut nonce = [0u8; TAM_NONCE];
        nonce.copy_from_slice(&bytes[37..49]);

        Ok(Self {
            version,
            salt,
            memory_kib,
            iterations,
            parallelism,
            hash_length,
            nonce,
        })
    }

    /// Serializa la cabecera en su representación binaria de 49 bytes en Big-Endian.
    pub fn a_bytes(&self) -> [u8; TAM_CABECERA] {
        let mut buf = [0u8; TAM_CABECERA];
        buf[0..4].copy_from_slice(MAGIC_BVDA);
        buf[4] = self.version;
        buf[5..21].copy_from_slice(&self.salt);
        BigEndian::write_u32(&mut buf[21..25], self.memory_kib);
        BigEndian::write_u32(&mut buf[25..29], self.iterations);
        BigEndian::write_u32(&mut buf[29..33], self.parallelism);
        BigEndian::write_u32(&mut buf[33..37], self.hash_length);
        buf[37..49].copy_from_slice(&self.nonce);
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cabecera_roundtrip() {
        let salt = [0x42u8; TAM_SALT];
        let nonce = [0x99u8; TAM_NONCE];
        let cabecera = CabeceraBvda::nueva(salt, 65_536, 3, 4, nonce).expect("cabecera válida");

        let bytes = cabecera.a_bytes();
        assert_eq!(bytes.len(), TAM_CABECERA);

        // Simulamos un archivo con cabecera + tag mínimo
        let mut archivo_simulado = Vec::new();
        archivo_simulado.extend_from_slice(&bytes);
        archivo_simulado.extend_from_slice(&[0u8; TAM_TAG_GCM]);

        let parseada = CabeceraBvda::parsear(&archivo_simulado).expect("debe parsear");
        assert_eq!(cabecera, parseada);
    }

    #[test]
    fn test_magic_invalido() {
        let mut bytes = [0u8; TAM_CABECERA + TAM_TAG_GCM];
        bytes[0..4].copy_from_slice(b"NOPE");
        assert!(matches!(CabeceraBvda::parsear(&bytes), Err(BvdaError::MagicoInvalido)));
    }

    #[test]
    fn test_parametros_fuera_de_rango() {
        let salt = [0u8; TAM_SALT];
        let nonce = [0u8; TAM_NONCE];
        // Memoria inferior a 1024 KiB
        assert!(matches!(
            CabeceraBvda::nueva(salt, 512, 3, 4, nonce),
            Err(BvdaError::ParametrosArgon2Invalidos)
        ));
        // Iteraciones fuera de rango (0)
        assert!(matches!(
            CabeceraBvda::nueva(salt, 65_536, 0, 4, nonce),
            Err(BvdaError::ParametrosArgon2Invalidos)
        ));
    }
}
