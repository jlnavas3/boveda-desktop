/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Micro-componente para compresión y descompresión transparente BLZ1 (Zlib RFC 1950).
//! Compatible al 100% con CompresorBoveda.kt de Bóveda Local Android.

use crate::crypto::error::BvdaError;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::{Read, Write};
use zeroize::Zeroizing;

pub const MAGIC_BLZ1: &[u8; 4] = b"BLZ1";

/// Comprime un búfer de datos utilizando Zlib (RFC 1950) con compresión máxima y prefijo "BLZ1".
pub fn comprimir(datos: &[u8]) -> Result<Vec<u8>, BvdaError> {
    let mut salida = Vec::with_capacity(datos.len() / 2 + 32);
    salida.extend_from_slice(MAGIC_BLZ1);

    let mut encoder = ZlibEncoder::new(salida, Compression::best());
    encoder
        .write_all(datos)
        .map_err(|e| BvdaError::ErrorDescompresion(e.to_string()))?;

    let resultado = encoder
        .finish()
        .map_err(|e| BvdaError::ErrorDescompresion(e.to_string()))?;

    Ok(resultado)
}

/// Descomprime un búfer de datos. Si contiene la marca "BLZ1", se descomprime vía Zlib;
/// si no la contiene, se devuelve el contenido tal cual para retrocompatibilidad histórica.
pub fn descomprimir(datos: &[u8]) -> Result<Zeroizing<Vec<u8>>, BvdaError> {
    if !es_comprimido(datos) {
        // Retrocompatibilidad con archivos antiguos sin comprimir
        return Ok(Zeroizing::new(datos.to_vec()));
    }

    let mut decoder = ZlibDecoder::new(&datos[MAGIC_BLZ1.len()..]);
    let mut buffer = Zeroizing::new(Vec::with_capacity(datos.len() * 2));

    decoder
        .read_to_end(&mut buffer)
        .map_err(|e| BvdaError::ErrorDescompresion(e.to_string()))?;

    Ok(buffer)
}

/// Comprueba si los datos inician con la cabecera mágica "BLZ1".
pub fn es_comprimido(datos: &[u8]) -> bool {
    datos.starts_with(MAGIC_BLZ1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compresion_descompresion_roundtrip() {
        let texto_original = b"Texto largo para probar compresion BLZ1 zlib repetitivo repetitivo repetitivo";
        let comprimido = comprimir(texto_original).expect("debe comprimir");

        assert!(es_comprimido(&comprimido));
        assert_eq!(&comprimido[0..4], MAGIC_BLZ1);

        let descomprimido = descomprimir(&comprimido).expect("debe descomprimir");
        assert_eq!(&*descomprimido, texto_original);
    }

    #[test]
    fn test_retrocompatibilidad_sin_comprimir() {
        let texto_plano = b"{\"version\":1,\"entradas\":[]}";
        assert!(!es_comprimido(texto_plano));

        let resultado = descomprimir(texto_plano).expect("debe procesar texto plano");
        assert_eq!(&*resultado, texto_plano);
    }
}
