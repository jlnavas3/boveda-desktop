/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Pruebas de integración del transporte óptico cifrado por QR (`bvda-opt://`).
//! Valida compatibilidad canónica con Bóveda Local Android (CodecDeltaOpticoTest.kt).

use boveda_core::models::{Categoria, ContenidoBoveda, Entrada};
use boveda_core::sync::{
    calcular_crc32_hex, es_uri_optica, EnsambladorRafagaOptica, TramaOpticaQr,
    MAX_BYTES_POR_CHUNK_DEFECTO, MAX_CHUNK_BYTES,
};

fn crear_boveda_prueba(num_entradas: usize) -> ContenidoBoveda {
    let cat = Categoria {
        id: "cat-personal".to_string(),
        nombre: "Personal".to_string(),
        ..Default::default()
    };

    let ahora = chrono::Utc::now().timestamp_millis();
    let entradas: Vec<Entrada> = (1..=num_entradas)
        .map(|i| Entrada {
            id: format!("id-{i}"),
            titulo: format!("Cuenta {i}"),
            usuario: format!("usuario{i}@correo.com"),
            contrasena: format!("claveSegura{i}"),
            categorias: vec![cat.id.clone()],
            modificada_en: ahora,
            creada_en: ahora,
            ..Default::default()
        })
        .collect();

    ContenidoBoveda {
        version: 1,
        entradas,
        papelera: Vec::new(),
        categorias: vec![cat],
        perfiles: Vec::new(),
        configuracion: None,
        alias_dispositivo: None,
    }
}

#[test]
fn test_empaquetar_y_desempaquetar_delta_estatico_1_qr() {
    let boveda = crear_boveda_prueba(2);
    // Permite verificar la generación y recepción de 1 único QR estático (1/1)
    let paquete = boveda_core::sync::empaquetar(&boveda, true, 600)
        .expect("Debe empaquetar delta");

    assert_eq!(paquete.total_tramas, 1);
    assert_eq!(paquete.tramas.len(), 1);
    assert_eq!(paquete.pin_efimero.len(), 6);
    assert_eq!(paquete.id_sesion_hex.len(), 8);

    let trama = &paquete.tramas[0];
    assert_eq!(trama.indice, 1);
    assert_eq!(trama.total, 1);

    // Verificar serialización y parseo de URI óptico
    let uri = trama.serializar_uri();
    assert!(uri.starts_with("bvda-opt://v1/1/1/"));
    assert!(es_uri_optica(&uri));

    let trama_parseada = TramaOpticaQr::parsear_uri(&uri).expect("Debe parsear URI");
    assert_eq!(trama, &trama_parseada);

    // Ensamblar y desempaquetar con EnsambladorRafagaOptica
    let mut ensamblador = EnsambladorRafagaOptica::new();
    let agregada = ensamblador.registrar_trama(&trama_parseada);
    assert!(agregada);
    assert!(ensamblador.esta_completo());

    let recuperada = ensamblador
        .descifrar_boveda(&paquete.pin_efimero)
        .expect("Debe descifrar bóveda");

    assert_eq!(recuperada.entradas.len(), 2);
    assert_eq!(recuperada.entradas[0].titulo, "Cuenta 1");
    assert_eq!(recuperada.entradas[0].usuario, "usuario1@correo.com");
    assert_eq!(recuperada.entradas[0].contrasena, "claveSegura1");
    assert_eq!(recuperada.entradas[1].titulo, "Cuenta 2");
}

#[test]
fn test_empaquetar_rafaga_fragmentada_y_reensamblar_desordenada_y_duplicados() {
    let boveda = crear_boveda_prueba(10);
    // Forzar fragmentación con tamaño pequeño de chunk (120 bytes)
    let paquete = boveda_core::sync::empaquetar(&boveda, false, 120)
        .expect("Debe empaquetar bóveda fragmentada");

    assert!(
        paquete.total_tramas >= 3,
        "Debe tener al menos 3 tramas, obtuvo {}",
        paquete.total_tramas
    );
    assert_eq!(paquete.total_tramas, paquete.tramas.len());

    let mut ensamblador = EnsambladorRafagaOptica::new();

    // Simular recepción desordenada y con tramas duplicadas:
    // Insertamos tramas en orden inverso y luego reenviamos la primera y la última
    let mut tramas_desordenadas = paquete.tramas.clone();
    tramas_desordenadas.reverse();

    // Replicar duplicados al inicio y final
    if let Some(primera) = paquete.tramas.first() {
        tramas_desordenadas.push(primera.clone());
    }
    if let Some(ultima) = paquete.tramas.last() {
        tramas_desordenadas.insert(0, ultima.clone());
    }

    for trama in &tramas_desordenadas {
        let uri = trama.serializar_uri();
        let parseada = TramaOpticaQr::parsear_uri(&uri).expect("Debe parsear");
        ensamblador.registrar_trama(&parseada);
    }

    assert!(ensamblador.esta_completo());
    assert!((ensamblador.porcentaje() - 1.0).abs() < 0.001);

    let recuperada = ensamblador
        .descifrar_boveda(&paquete.pin_efimero)
        .expect("Debe descifrar con éxito");
    assert_eq!(recuperada.entradas.len(), 10);
}

#[test]
fn test_ergonomia_max_chunk_bytes_300() {
    let boveda = crear_boveda_prueba(2);
    // Verificar que el tamaño por defecto sea 300 bytes y todos los chunks cumplan el límite
    assert_eq!(MAX_CHUNK_BYTES, 300);
    assert_eq!(MAX_BYTES_POR_CHUNK_DEFECTO, 300);

    let paquete = boveda_core::sync::empaquetar(&boveda, false, MAX_CHUNK_BYTES)
        .expect("Debe empaquetar");

    for trama in &paquete.tramas {
        assert!(
            trama.datos_chunk.len() <= MAX_CHUNK_BYTES,
            "Chunk excede 300 bytes: {}",
            trama.datos_chunk.len()
        );
    }

    let mut ensamblador = EnsambladorRafagaOptica::new();
    for trama in &paquete.tramas {
        ensamblador.registrar_trama(trama);
    }
    assert!(ensamblador.esta_completo());
    let recuperada = ensamblador
        .descifrar_boveda(&paquete.pin_efimero)
        .expect("Debe descifrar bóveda");
    assert_eq!(recuperada.entradas.len(), 2);
}

#[test]
fn test_rechazo_con_pin_incorrecto() {
    let boveda = crear_boveda_prueba(1);
    let paquete = boveda_core::sync::empaquetar(&boveda, true, MAX_CHUNK_BYTES)
        .expect("Debe empaquetar");

    let pin_incorrecto = if paquete.pin_efimero == "123456" {
        "654321"
    } else {
        "123456"
    };

    let mut ensamblador = EnsambladorRafagaOptica::new();
    for trama in &paquete.tramas {
        ensamblador.registrar_trama(trama);
    }

    let resultado = ensamblador.descifrar_boveda(pin_incorrecto);
    assert!(resultado.is_err());
    let err_msg = resultado.unwrap_err();
    assert!(
        err_msg.contains("PIN óptico de 6 dígitos incorrecto"),
        "Mensaje inesperado: {err_msg}"
    );
}

#[test]
fn test_rechazo_con_crc32_corrupto() {
    let boveda = crear_boveda_prueba(1);
    let paquete = boveda_core::sync::empaquetar(&boveda, true, MAX_CHUNK_BYTES)
        .expect("Debe empaquetar");

    let mut ensamblador = EnsambladorRafagaOptica::new();
    for (i, trama) in paquete.tramas.iter().enumerate() {
        if i == 0 {
            let mut datos_corruptos = trama.datos_chunk.clone();
            datos_corruptos[0] ^= 0xFF;
            let trama_corrupta = TramaOpticaQr {
                version: trama.version,
                indice: trama.indice,
                total: trama.total,
                id_sesion_hex: trama.id_sesion_hex.clone(),
                crc32_total_hex: trama.crc32_total_hex.clone(),
                datos_chunk: datos_corruptos,
                alias_emisor: trama.alias_emisor.clone(),
            };
            ensamblador.registrar_trama(&trama_corrupta);
        } else {
            ensamblador.registrar_trama(trama);
        }
    }

    let resultado = ensamblador.descifrar_boveda(&paquete.pin_efimero);
    assert!(resultado.is_err());
    let err_msg = resultado.unwrap_err();
    assert!(
        err_msg.contains("Fallo de integridad CRC32"),
        "Mensaje inesperado: {err_msg}"
    );
}

#[test]
fn test_validacion_uri_y_esquema() {
    assert!(!es_uri_optica(""));
    assert!(!es_uri_optica("https://ejemplo.com"));
    assert!(!es_uri_optica("bvda-sync://192.168.1.5:8080"));
    assert!(es_uri_optica("bvda-opt://v1/1/1/A1B2C3D4/E5A3019F?d=SGVsbG8"));

    // Casos inválidos para parsear
    assert!(TramaOpticaQr::parsear_uri("bvda-opt://invalid").is_none());
    assert!(TramaOpticaQr::parsear_uri("bvda-opt://v1/2/1/A1B2C3D4/E5A3019F?d=SGVsbG8").is_none()); // indice > total
    assert!(TramaOpticaQr::parsear_uri("bvda-opt://v1/0/1/A1B2C3D4/E5A3019F?d=SGVsbG8").is_none()); // indice < 1
    assert!(TramaOpticaQr::parsear_uri("bvda-opt://v1/1/1/A1B2/E5A3019F?d=SGVsbG8").is_none()); // id_sesion != 8 chars
    assert!(TramaOpticaQr::parsear_uri("bvda-opt://v1/1/1/A1B2C3D4/E5A3?d=SGVsbG8").is_none()); // crc != 8 chars
    assert!(TramaOpticaQr::parsear_uri("bvda-opt://v1/1/1/A1B2C3D4/E5A3019F").is_none()); // sin ?d=
    assert!(TramaOpticaQr::parsear_uri("bvda-opt://v1/1/1/A1B2C3D4/E5A3019F?d=").is_none()); // ?d= vacío

    // Cálculo directo de CRC32
    let crc = calcular_crc32_hex(b"123456789");
    assert_eq!(crc, "CBF43926"); // Vector canónico estándar de CRC32 para "123456789"
}

#[test]
fn test_trama_optica_alias_emisor() {
    let uri_con_alias = "bvda-opt://v1/1/2/A1B2C3D4/E5A3019F?d=SGVsbG8&a=Honor%20Magic%206%20Pro";
    let trama = TramaOpticaQr::parsear_uri(uri_con_alias).expect("Debe parsear");
    assert_eq!(trama.alias_emisor.as_deref(), Some("Honor Magic 6 Pro"));

    let serializada = trama.serializar_uri();
    assert!(serializada.contains("&a=Honor%20Magic%206%20Pro"));

    let mut ensamblador = EnsambladorRafagaOptica::new();
    assert_eq!(ensamblador.alias_emisor(), None);
    ensamblador.registrar_trama(&trama);
    assert_eq!(ensamblador.alias_emisor(), Some("Honor Magic 6 Pro"));

    ensamblador.reiniciar();
    assert_eq!(ensamblador.alias_emisor(), None);
}
