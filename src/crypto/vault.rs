/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Orquestación de alto nivel del ciclo de vida del contenedor `.bvda`:
//! Desempaquetado, empaquetado, persistencia atómica y zeroización en memoria.

use crate::crypto::cipher::{cifrar_aes_gcm, descifrar_aes_gcm};
use crate::crypto::compressor::{comprimir, descomprimir};
use crate::crypto::error::BvdaError;
use crate::crypto::header::{CabeceraBvda, TAM_CABECERA, TAM_NONCE, TAM_SALT};
use crate::crypto::kdf::{derivar_clave, KdfParams, PerfilArgon2};
use crate::models::contenido::ContenidoBoveda;
use rand::RngCore;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

/// Desempaqueta y descifra un archivo binario `.bvda` en su estructura de dominio `ContenidoBoveda`.
/// Todas las claves derivadas y buffers planos se destruyen de la memoria RAM al finalizar.
pub fn desempaquetar_boveda(
    bytes_archivo: &[u8],
    password: &[u8],
) -> Result<ContenidoBoveda, BvdaError> {
    let cabecera = CabeceraBvda::parsear(bytes_archivo)?;
    let aad = &bytes_archivo[..TAM_CABECERA];
    let cuerpo_cifrado = &bytes_archivo[TAM_CABECERA..];

    let params = KdfParams {
        memory_kib: cabecera.memory_kib,
        iterations: cabecera.iterations,
        parallelism: cabecera.parallelism,
        hash_length: cabecera.hash_length,
    };

    let clave = derivar_clave(password, &cabecera.salt, &params)?;
    let bytes_comprimidos = descifrar_aes_gcm(cuerpo_cifrado, &clave, &cabecera.nonce, aad)?;
    let json_bytes = descomprimir(&bytes_comprimidos)?;
    let contenido: ContenidoBoveda = serde_json::from_slice(&json_bytes)?;

    Ok(contenido)
}

/// Cifra y empaqueta la estructura `ContenidoBoveda` generando un archivo `.bvda` compatible con Android.
pub fn empaquetar_boveda(
    contenido: &ContenidoBoveda,
    password: &[u8],
    perfil: PerfilArgon2,
) -> Result<Vec<u8>, BvdaError> {
    let json_bytes = serde_json::to_vec(contenido)?;
    let bytes_comprimidos = comprimir(&json_bytes)?;

    let mut salt = [0u8; TAM_SALT];
    let mut nonce = [0u8; TAM_NONCE];
    let mut rng = rand::thread_rng();
    rng.fill_bytes(&mut salt);
    rng.fill_bytes(&mut nonce);

    let kdf_params = perfil.a_kdf_params();
    let cabecera = CabeceraBvda::nueva(
        salt,
        kdf_params.memory_kib,
        kdf_params.iterations,
        kdf_params.parallelism,
        nonce,
    )?;

    let cabecera_bytes = cabecera.a_bytes();
    let clave = derivar_clave(password, &salt, &kdf_params)?;
    let cuerpo_cifrado = cifrar_aes_gcm(&bytes_comprimidos, &clave, &nonce, &cabecera_bytes)?;

    let mut archivo_completo = Vec::with_capacity(TAM_CABECERA + cuerpo_cifrado.len());
    archivo_completo.extend_from_slice(&cabecera_bytes);
    archivo_completo.extend_from_slice(&cuerpo_cifrado);

    Ok(archivo_completo)
}

/// Escribe un archivo de forma atómica y segura para evitar corrupción por caídas del sistema
/// (escribe en .tmp con fsync y luego renombra).
pub fn escribir_atomico(destino: &Path, contenido: &[u8]) -> Result<(), BvdaError> {
    if let Some(padre) = destino.parent() {
        fs::create_dir_all(padre)?;
    }

    let nombre_temporal = format!(
        "{}.tmp",
        destino
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("boveda")
    );
    let ruta_temporal = destino.with_file_name(nombre_temporal);

    {
        let mut f = File::create(&ruta_temporal)?;
        f.write_all(contenido)?;
        f.flush()?;
        f.sync_all()?;
    }

    fs::rename(&ruta_temporal, destino)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::entrada::Entrada;
    use crate::models::tarjeta::DatosTarjetaBancaria;
    use crate::models::tipo_entrada::TipoEntrada;

    #[test]
    fn test_ciclo_completo_empaquetar_desempaquetar() {
        let mut boveda = ContenidoBoveda::default();
        boveda.entradas.push(Entrada {
            id: "e-test-1".into(),
            titulo: "Cuenta Principal".into(),
            usuario: "admin@local".into(),
            contrasena: "claveSuperSecreta2026!".into(),
            tipo: TipoEntrada::Login,
            favorito: true,
            ..Default::default()
        });

        boveda.entradas.push(Entrada {
            id: "e-test-2".into(),
            titulo: "Tarjeta Débito".into(),
            tipo: TipoEntrada::Tarjeta,
            datos_tarjeta: Some(DatosTarjetaBancaria {
                titular: "Jose Luis".into(),
                numero: "4532000011112222".into(),
                vencimiento: "10/30".into(),
                cvv: "999".into(),
                ..Default::default()
            }),
            ..Default::default()
        });

        let password = b"mi_contrasena_maestra";
        // Usamos KdfParams ligeros en test mediante función interna o perfil
        let archivo_bvda = empaquetar_boveda(&boveda, password, PerfilArgon2::Estandar)
            .expect("empaquetar debe funcionar");

        // 1. Debe empezar con "BVDA"
        assert_eq!(&archivo_bvda[0..4], b"BVDA");

        // 2. Desempaquetado correcto con contraseña buena
        let recuperada = desempaquetar_boveda(&archivo_bvda, password)
            .expect("desempaquetar debe funcionar");
        assert_eq!(recuperada.entradas.len(), 2);
        assert_eq!(recuperada.entradas[0].titulo, "Cuenta Principal");
        assert_eq!(recuperada.entradas[1].titulo, "Tarjeta Débito");
        assert_eq!(
            recuperada.entradas[1].datos_tarjeta.as_ref().unwrap().numero,
            "4532000011112222"
        );

        // 3. Fallo con contraseña mala
        let resultado_malo = desempaquetar_boveda(&archivo_bvda, b"contrasena_incorrecta");
        assert!(matches!(resultado_malo, Err(BvdaError::AutenticacionFallida)));
    }

    #[test]
    fn test_escritura_atomica() {
        let temp_dir = std::env::temp_dir().join(format!("bvda-test-{}", std::process::id()));
        let archivo = temp_dir.join("test.bvda");
        let contenido = b"datos binarios seguros";

        escribir_atomico(&archivo, contenido).expect("escritura atomica exitosa");
        assert!(archivo.exists());

        let leido = fs::read(&archivo).expect("lectura exitosa");
        assert_eq!(leido, contenido);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
