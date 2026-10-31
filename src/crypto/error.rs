/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Definición de errores criptográficos y del formato .bvda.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum BvdaError {
    #[error("Archivo demasiado corto para contener una cabecera válida")]
    ArchivoDemasiadoCorto,
    #[error("Identificador mágico incorrecto (esperado 'BVDA')")]
    MagicoInvalido,
    #[error("Versión de archivo .bvda no soportada: {0}")]
    VersionNoSoportada(u8),
    #[error("Parámetros Argon2id fuera de rango")]
    ParametrosArgon2Invalidos,
    #[error("Contraseña incorrecta o archivo dañado (fallo de autenticación GCM)")]
    AutenticacionFallida,
    #[error("Error de descompresión BLZ1/Zlib: {0}")]
    ErrorDescompresion(String),
    #[error("Error de deserialización JSON: {0}")]
    ErrorJson(#[from] serde_json::Error),
    #[error("Error de entrada/salida: {0}")]
    ErrorIo(#[from] std::io::Error),
    #[error("Error en configuración: {0}")]
    ErrorConfig(String),
    #[error("Registro o entrada no encontrada: {0}")]
    RegistroNoEncontrado(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = BvdaError::AutenticacionFallida;
        assert!(err.to_string().contains("Contraseña incorrecta"));

        let err_ver = BvdaError::VersionNoSoportada(99);
        assert!(err_ver.to_string().contains("99"));
    }
}
