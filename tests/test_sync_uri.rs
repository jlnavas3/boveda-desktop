/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

use boveda_core::sync::{
    es_ip_privada_rfc1918, es_uri_sync, parsear_sync_uri, serializar_sync_uri,
};

const K_HEX_CANON: &str = "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20";
const T_HEX_CANON: &str = "a1a2a3a4a5a6a7a8a9aaabacadaeafb0b1b2b3b4b5b6b7b8b9babbbcbdbebfc0";
const HUELLA_ESPERADA: &str = "74:5A:63:02:1A:A5";

#[test]
fn test_vector_canonico_p2p() {
    let uri = format!(
        "bvda-sync://192.168.1.50,10.0.0.15:54321?k={K_HEX_CANON}&t={T_HEX_CANON}&v=2"
    );

    assert!(es_uri_sync(&uri));

    let payload = parsear_sync_uri(&uri).expect("El URI canónico debe parsearse con éxito");
    assert_eq!(payload.ips, vec!["192.168.1.50", "10.0.0.15"]);
    assert_eq!(payload.puerto, 54321);
    assert_eq!(payload.version, 2);
    assert_eq!(payload.clave_publica_hex(), K_HEX_CANON);
    assert_eq!(payload.token_psk_hex(), T_HEX_CANON);

    // Verificación exacta de la Huella Visual Determinista
    assert_eq!(payload.huella_visual(), HUELLA_ESPERADA);

    // Roundtrip de serialización
    let serializado = serializar_sync_uri(&payload);
    assert_eq!(serializado, uri);
}

#[test]
fn test_rechazo_ip_publica_en_lista() {
    let uri = format!(
        "bvda-sync://192.168.1.50,8.8.8.8:54321?k={K_HEX_CANON}&t={T_HEX_CANON}&v=2"
    );
    assert!(parsear_sync_uri(&uri).is_err());
}

#[test]
fn test_rechazo_dominio_o_dns() {
    let uri = format!(
        "bvda-sync://mi-desktop.local:54321?k={K_HEX_CANON}&t={T_HEX_CANON}&v=2"
    );
    assert!(parsear_sync_uri(&uri).is_err());

    let uri_host = format!(
        "bvda-sync://localhost:54321?k={K_HEX_CANON}&t={T_HEX_CANON}&v=2"
    );
    assert!(parsear_sync_uri(&uri_host).is_err());
}

#[test]
fn test_rechazo_loopback() {
    let uri = format!(
        "bvda-sync://127.0.0.1:54321?k={K_HEX_CANON}&t={T_HEX_CANON}&v=2"
    );
    assert!(parsear_sync_uri(&uri).is_err());
}

#[test]
fn test_rechazo_puertos_fuera_de_rango() {
    // Puerto privilegiado (< 1024)
    let uri_privilegiado = format!(
        "bvda-sync://192.168.1.50:80?k={K_HEX_CANON}&t={T_HEX_CANON}&v=2"
    );
    assert!(parsear_sync_uri(&uri_privilegiado).is_err());

    // Puerto 1023
    let uri_1023 = format!(
        "bvda-sync://192.168.1.50:1023?k={K_HEX_CANON}&t={T_HEX_CANON}&v=2"
    );
    assert!(parsear_sync_uri(&uri_1023).is_err());

    // Puerto inválido (0)
    let uri_0 = format!(
        "bvda-sync://192.168.1.50:0?k={K_HEX_CANON}&t={T_HEX_CANON}&v=2"
    );
    assert!(parsear_sync_uri(&uri_0).is_err());
}

#[test]
fn test_rechazo_parametros_criptograficos_invalidos() {
    // k con 63 caracteres (longitud inválida)
    let uri_corta = format!(
        "bvda-sync://192.168.1.50:54321?k={}&t={T_HEX_CANON}&v=2",
        &K_HEX_CANON[1..]
    );
    assert!(parsear_sync_uri(&uri_corta).is_err());

    // t con caracteres no hexadecimales
    let t_invalida = format!("{}ZZ", &T_HEX_CANON[..62]);
    let uri_no_hex = format!(
        "bvda-sync://192.168.1.50:54321?k={K_HEX_CANON}&t={t_invalida}&v=2"
    );
    assert!(parsear_sync_uri(&uri_no_hex).is_err());
}

#[test]
fn test_rechazo_version_incompatible() {
    let uri_v1 = format!(
        "bvda-sync://192.168.1.50:54321?k={K_HEX_CANON}&t={T_HEX_CANON}&v=1"
    );
    assert!(parsear_sync_uri(&uri_v1).is_err());
}

#[test]
fn test_validador_rfc1918_rangos() {
    // Clase A (10.0.0.0/8)
    assert!(es_ip_privada_rfc1918("10.0.0.1"));
    assert!(es_ip_privada_rfc1918("10.255.255.254"));
    assert!(!es_ip_privada_rfc1918("9.255.255.255"));
    assert!(!es_ip_privada_rfc1918("11.0.0.1"));

    // Clase B (172.16.0.0/12)
    assert!(es_ip_privada_rfc1918("172.16.0.1"));
    assert!(es_ip_privada_rfc1918("172.31.255.254"));
    assert!(!es_ip_privada_rfc1918("172.15.255.255"));
    assert!(!es_ip_privada_rfc1918("172.32.0.1"));

    // Clase C (192.168.0.0/16)
    assert!(es_ip_privada_rfc1918("192.168.0.1"));
    assert!(es_ip_privada_rfc1918("192.168.1.50"));
    assert!(!es_ip_privada_rfc1918("192.167.1.1"));
    assert!(!es_ip_privada_rfc1918("192.169.1.1"));

    // Descarte de ceros a la izquierda (evitar interpretación octal)
    assert!(!es_ip_privada_rfc1918("010.0.0.1"));
    assert!(!es_ip_privada_rfc1918("192.168.01.1"));
    assert!(!es_ip_privada_rfc1918("172.016.1.1"));

    // Descarte de públicas, link-local, loopback
    assert!(!es_ip_privada_rfc1918("8.8.8.8"));
    assert!(!es_ip_privada_rfc1918("127.0.0.1"));
    assert!(!es_ip_privada_rfc1918("169.254.1.1"));
    assert!(!es_ip_privada_rfc1918("224.0.0.1"));
    assert!(!es_ip_privada_rfc1918("255.255.255.255"));
    assert!(!es_ip_privada_rfc1918("localhost"));
}
