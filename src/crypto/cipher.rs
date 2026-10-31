/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Cifrado y descifrado autenticado AES-256-GCM con AAD (Additional Authenticated Data).

use crate::crypto::error::BvdaError;
use aes_gcm::aead::{Aead, Payload};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use zeroize::Zeroizing;

/// Cifra un bloque de datos en claro utilizando AES-256-GCM y los datos asociados autenticados (AAD).
/// Retorna el texto cifrado concatenado con la etiqueta de autenticación (Tag) de 16 bytes.
pub fn cifrar_aes_gcm(
    plano: &[u8],
    clave: &[u8; 32],
    nonce: &[u8; 12],
    aad: &[u8],
) -> Result<Vec<u8>, BvdaError> {
    let cipher = Aes256Gcm::new_from_slice(clave)
        .map_err(|_| BvdaError::AutenticacionFallida)?;
    let nonce_obj = Nonce::from_slice(nonce);

    let payload = Payload {
        msg: plano,
        aad,
    };

    cipher
        .encrypt(nonce_obj, payload)
        .map_err(|_| BvdaError::AutenticacionFallida)
}

/// Descifra un bloque cifrado con etiqueta GCM de 16 bytes utilizando AES-256-GCM y AAD.
/// Retorna los bytes planos envueltos en `Zeroizing<Vec<u8>>` para limpieza segura al destruirse.
pub fn descifrar_aes_gcm(
    cifrado_con_tag: &[u8],
    clave: &[u8; 32],
    nonce: &[u8; 12],
    aad: &[u8],
) -> Result<Zeroizing<Vec<u8>>, BvdaError> {
    let cipher = Aes256Gcm::new_from_slice(clave)
        .map_err(|_| BvdaError::AutenticacionFallida)?;
    let nonce_obj = Nonce::from_slice(nonce);

    let payload = Payload {
        msg: cifrado_con_tag,
        aad,
    };

    let bytes = cipher
        .decrypt(nonce_obj, payload)
        .map_err(|_| BvdaError::AutenticacionFallida)?;

    Ok(Zeroizing::new(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cifrar_descifrar_con_aad() {
        let clave = [0x55u8; 32];
        let nonce = [0x77u8; 12];
        let aad = b"Cabecera Inmutable de 49 Bytes";
        let mensaje = b"Secreto confidencial en claro";

        let cifrado = cifrar_aes_gcm(mensaje, &clave, &nonce, aad).expect("cifrado exitoso");
        assert_eq!(cifrado.len(), mensaje.len() + 16);

        let descifrado = descifrar_aes_gcm(&cifrado, &clave, &nonce, aad).expect("descifrado exitoso");
        assert_eq!(&*descifrado, mensaje);
    }

    #[test]
    fn test_falla_si_aad_modificado() {
        let clave = [0x55u8; 32];
        let nonce = [0x77u8; 12];
        let aad_original = b"Cabecera AAD Original";
        let aad_adulterado = b"Cabecera AAD Alterada";
        let mensaje = b"Secreto confidencial";

        let cifrado = cifrar_aes_gcm(mensaje, &clave, &nonce, aad_original).expect("cifrado exitoso");

        let resultado = descifrar_aes_gcm(&cifrado, &clave, &nonce, aad_adulterado);
        assert!(matches!(resultado, Err(BvdaError::AutenticacionFallida)));
    }

    #[test]
    fn test_falla_si_cuerpo_modificado() {
        let clave = [0x55u8; 32];
        let nonce = [0x77u8; 12];
        let aad = b"Cabecera AAD";
        let mensaje = b"Secreto";

        let mut cifrado = cifrar_aes_gcm(mensaje, &clave, &nonce, aad).expect("cifrado exitoso");
        let ult = cifrado.len() - 1;
        cifrado[ult] ^= 0x01; // Corromper el tag

        let resultado = descifrar_aes_gcm(&cifrado, &clave, &nonce, aad);
        assert!(matches!(resultado, Err(BvdaError::AutenticacionFallida)));
    }
}
