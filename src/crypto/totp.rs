/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Generador y validador de códigos de un solo uso basados en tiempo (TOTP RFC 6238).
//! Compatible al 100% con la implementación de Android y estándares RFC 4226 / RFC 6238 / RFC 4648.

use hmac::{Hmac, Mac};
use sha1::Sha1;

type HmacSha1 = Hmac<Sha1>;

const ALFABETO_BASE32: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
pub const PERIODO_DEFECTO: u64 = 30;
pub const DIGITOS_DEFECTO: usize = 6;

/// Decodifica una cadena Base32 RFC 4648 a bytes, ignorando espacios, guiones y signos '=' finales.
pub fn decodificar_base32(texto: &str) -> Result<Vec<u8>, String> {
    let limpio = texto
        .to_uppercase()
        .replace([' ', '-'], "")
        .trim_end_matches('=')
        .to_string();

    if limpio.is_empty() {
        return Err("Secreto Base32 vacío".into());
    }

    let mut buffer: u32 = 0;
    let mut bits: u8 = 0;
    let mut salida = Vec::with_capacity(limpio.len() * 5 / 8 + 1);

    for c in limpio.chars() {
        let indice = ALFABETO_BASE32
            .find(c)
            .ok_or_else(|| format!("Carácter Base32 no válido: '{c}'"))? as u32;
        buffer = (buffer << 5) | indice;
        bits += 5;

        if bits >= 8 {
            bits -= 8;
            salida.push(((buffer >> bits) & 0xFF) as u8);
        }
    }

    if salida.is_empty() {
        return Err("Secreto Base32 demasiado corto".into());
    }

    Ok(salida)
}

/// Codifica bytes a una cadena Base32 estándar sin padding.
pub fn codificar_base32(bytes: &[u8]) -> String {
    let mut resultado = String::new();
    let mut buffer: u32 = 0;
    let mut bits: u8 = 0;

    for &b in bytes {
        buffer = (buffer << 8) | (b as u32);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            let idx = ((buffer >> bits) & 0x1F) as usize;
            resultado.push(ALFABETO_BASE32.chars().nth(idx).unwrap_or('A'));
        }
    }

    if bits > 0 {
        let idx = ((buffer << (5 - bits)) & 0x1F) as usize;
        resultado.push(ALFABETO_BASE32.chars().nth(idx).unwrap_or('A'));
    }

    resultado
}

/// Calcula el valor HOTP (RFC 4226) para un contador y secreto binario dado.
pub fn hotp(secreto: &[u8], contador: u64, digitos: usize) -> Result<String, String> {
    let mut mac = HmacSha1::new_from_slice(secreto)
        .map_err(|e| format!("Error inicializando HMAC-SHA1: {e}"))?;

    let mensaje = contador.to_be_bytes();
    mac.update(&mensaje);
    let hash = mac.finalize().into_bytes();

    let desplazamiento = (hash[hash.len() - 1] & 0x0F) as usize;
    let binario = (((hash[desplazamiento] & 0x7F) as u32) << 24)
        | ((hash[desplazamiento + 1] as u32) << 16)
        | ((hash[desplazamiento + 2] as u32) << 8)
        | (hash[desplazamiento + 3] as u32);

    let mut modulo: u32 = 1;
    for _ in 0..digitos {
        modulo = modulo.saturating_mul(10);
    }

    let codigo = binario % modulo;
    Ok(format!("{:0width$}", codigo, width = digitos))
}

/// Calcula el código TOTP (RFC 6238) para un secreto binario y momento Unix en segundos.
pub fn codigo_totp(
    secreto: &[u8],
    segundos_unix: u64,
    digitos: usize,
    periodo: u64,
) -> Result<String, String> {
    let periodo_efectivo = if periodo == 0 { PERIODO_DEFECTO } else { periodo };
    let contador = segundos_unix / periodo_efectivo;
    hotp(secreto, contador, digitos)
}

/// Calcula el código TOTP a partir de un secreto Base32.
pub fn codigo_totp_desde_base32(
    secreto_base32: &str,
    segundos_unix: u64,
    digitos: usize,
    periodo: u64,
) -> Result<String, String> {
    let secreto = decodificar_base32(secreto_base32)?;
    codigo_totp(&secreto, segundos_unix, digitos, periodo)
}

/// Retorna los segundos restantes dentro de la ventana de tiempo actual (ej. 30 a 1).
pub fn segundos_restantes_totp(segundos_unix: u64, periodo: u64) -> u64 {
    let periodo_efectivo = if periodo == 0 { PERIODO_DEFECTO } else { periodo };
    periodo_efectivo - (segundos_unix % periodo_efectivo)
}

/// Formatea un código de 6 u 8 dígitos con espacio visual en el medio (ej. "123 456").
pub fn formatear_codigo_totp(codigo: &str) -> String {
    let limpio = codigo.trim();
    if limpio.len() == 6 {
        format!("{} {}", &limpio[..3], &limpio[3..])
    } else if limpio.len() == 8 {
        format!("{} {}", &limpio[..4], &limpio[4..])
    } else {
        limpio.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vectores_oficiales_rfc_6238() {
        let secreto = b"12345678901234567890";

        assert_eq!(codigo_totp(secreto, 59, 8, 30).unwrap(), "94287082");
        assert_eq!(
            codigo_totp(secreto, 1_111_111_109, 8, 30).unwrap(),
            "07081804"
        );
        assert_eq!(
            codigo_totp(secreto, 1_111_111_111, 8, 30).unwrap(),
            "14050471"
        );
        assert_eq!(
            codigo_totp(secreto, 1_234_567_890, 8, 30).unwrap(),
            "89005924"
        );
        assert_eq!(
            codigo_totp(secreto, 2_000_000_000, 8, 30).unwrap(),
            "69279037"
        );
        assert_eq!(
            codigo_totp(secreto, 20_000_000_000, 8, 30).unwrap(),
            "65353130"
        );
    }

    #[test]
    fn test_codigo_seis_digitos_y_formateo() {
        let secreto = b"12345678901234567890";
        let codigo_6 = codigo_totp(secreto, 59, 6, 30).unwrap();
        assert_eq!(codigo_6, "287082");
        assert_eq!(formatear_codigo_totp(&codigo_6), "287 082");
    }

    #[test]
    fn test_totp_desde_base32() {
        // Mismo vector que en Android TotpTest.kt
        let codigo = codigo_totp_desde_base32("JBSWY3DPEHPK3PXP", 1_000_000, 6, 30).unwrap();
        assert_eq!(codigo, "346137");

        // Espacios y guiones deben ser tolerados limpiamente
        let codigo_espacios =
            codigo_totp_desde_base32("JBSW-Y3DP EHPK 3PXP===", 1_000_000, 6, 30).unwrap();
        assert_eq!(codigo_espacios, "346137");
    }

    #[test]
    fn test_segundos_restantes() {
        assert_eq!(segundos_restantes_totp(60, 30), 30);
        assert_eq!(segundos_restantes_totp(89, 30), 1);
        assert_eq!(segundos_restantes_totp(90, 30), 30);
    }
}
