/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

use boveda_core::sync::{
    calcular_diffie_hellman, construir_client_hello, derivar_clave_publica_x25519,
    derivar_claves_sesion, responder_server_hello, verificar_server_hello, SessionCipher,
};

fn a_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[test]
fn test_vector_de_oro_handshake_y_derivacion() {
    let sk_desktop: [u8; 32] = [0x11; 32];
    let sk_android: [u8; 32] = [0x22; 32];
    let psk_token: [u8; 32] = [0x33; 32];

    let pk_desktop = derivar_clave_publica_x25519(&sk_desktop);
    let pk_android = derivar_clave_publica_x25519(&sk_android);

    // 1. Simetría matemática de Diffie-Hellman X25519
    let dh_desktop = calcular_diffie_hellman(&sk_desktop, &pk_android);
    let dh_android = calcular_diffie_hellman(&sk_android, &pk_desktop);
    assert_eq!(dh_desktop, dh_android, "El secreto DH X25519 debe ser idéntico en ambos extremos");

    // 2. Derivación de claves de sesión HKDF-SHA256
    let keys_desktop = derivar_claves_sesion(&dh_desktop, &psk_token, &pk_desktop, &pk_android).unwrap();
    let keys_android = derivar_claves_sesion(&dh_android, &psk_token, &pk_desktop, &pk_android).unwrap();

    assert_eq!(keys_desktop.key_a2d, keys_android.key_a2d);
    assert_eq!(keys_desktop.key_d2a, keys_android.key_d2a);
    assert_eq!(keys_desktop.handshake_key, keys_android.handshake_key);

    println!("Vector de Oro - pk_desktop:     {}", a_hex(&pk_desktop));
    println!("Vector de Oro - pk_android:     {}", a_hex(&pk_android));
    println!("Vector de Oro - dh_shared:      {}", a_hex(&dh_desktop));
    println!("Vector de Oro - key_a2d:        {}", a_hex(&keys_desktop.key_a2d));
    println!("Vector de Oro - key_d2a:        {}", a_hex(&keys_desktop.key_d2a));
    println!("Vector de Oro - handshake_key:  {}", a_hex(&keys_desktop.handshake_key));

    // 3. Handshake 1-RTT: Android genera ClientHello (64 bytes)
    let (client_hello, client_keys) = construir_client_hello(&sk_android, &pk_desktop, &psk_token).unwrap();
    assert_eq!(client_hello.len(), 64);
    assert_eq!(&client_hello[0..32], &pk_android[..]);
    assert_eq!(client_keys.key_a2d, keys_android.key_a2d);

    // 4. Desktop procesa ClientHello y responde con ServerHello (32 bytes)
    let (server_hello, server_keys) = responder_server_hello(&sk_desktop, &psk_token, &client_hello).unwrap();
    assert_eq!(server_hello.len(), 32);
    assert_eq!(server_keys.key_a2d, keys_desktop.key_a2d);
    assert_eq!(server_keys.key_d2a, keys_desktop.key_d2a);

    println!("Vector de Oro - ServerHello:    {}", a_hex(&server_hello));

    // 5. Android verifica ServerHello
    let verif = verificar_server_hello(
        &client_keys.handshake_key,
        &pk_desktop,
        &pk_android,
        &server_hello,
    );
    assert!(verif.is_ok(), "Android debe verificar ServerHello exitosamente");
}

#[test]
fn test_fallo_inmediato_psk_alterada() {
    let sk_desktop: [u8; 32] = [0x11; 32];
    let sk_android: [u8; 32] = [0x22; 32];
    let psk_token_correcto: [u8; 32] = [0x33; 32];

    // Android usa PSK correcta
    let (client_hello, _) = construir_client_hello(
        &sk_android,
        &derivar_clave_publica_x25519(&sk_desktop),
        &psk_token_correcto,
    )
    .unwrap();

    // Desktop tiene 1 bit alterado en PSK (ej. Man-in-the-Middle o PIN QR erróneo)
    let mut psk_alterada = psk_token_correcto;
    psk_alterada[0] ^= 0x01;

    let res = responder_server_hello(&sk_desktop, &psk_alterada, &client_hello);
    assert!(
        res.is_err(),
        "Desktop debe rechazar de inmediato ClientHello si la PSK difiere"
    );
}

#[test]
fn test_fallo_inmediato_client_hello_tamper() {
    let sk_desktop: [u8; 32] = [0x11; 32];
    let sk_android: [u8; 32] = [0x22; 32];
    let psk_token: [u8; 32] = [0x33; 32];

    let (mut client_hello, _) = construir_client_hello(
        &sk_android,
        &derivar_clave_publica_x25519(&sk_desktop),
        &psk_token,
    )
    .unwrap();

    // Alterar 1 bit en la etiqueta HMAC
    client_hello[63] ^= 0x80;

    let res = responder_server_hello(&sk_desktop, &psk_token, &client_hello);
    assert!(res.is_err(), "Desktop debe detectar alteración de integridad en ClientHello");
}

#[test]
fn test_fallo_inmediato_server_hello_tamper() {
    let sk_desktop: [u8; 32] = [0x11; 32];
    let sk_android: [u8; 32] = [0x22; 32];
    let psk_token: [u8; 32] = [0x33; 32];

    let pk_desktop = derivar_clave_publica_x25519(&sk_desktop);
    let (client_hello, client_keys) = construir_client_hello(&sk_android, &pk_desktop, &psk_token).unwrap();
    let (mut server_hello, _) = responder_server_hello(&sk_desktop, &psk_token, &client_hello).unwrap();

    let pk_android = derivar_clave_publica_x25519(&sk_android);

    // Alterar 1 bit de ServerHello
    server_hello[15] ^= 0x01;

    let res = verificar_server_hello(&client_keys.handshake_key, &pk_desktop, &pk_android, &server_hello);
    assert!(res.is_err(), "Android debe rechazar ServerHello alterado");
}

#[test]
fn test_tunel_bidireccional_aead_y_secuencia() {
    let sk_desktop: [u8; 32] = [0x11; 32];
    let sk_android: [u8; 32] = [0x22; 32];
    let psk_token: [u8; 32] = [0x33; 32];

    let pk_desktop = derivar_clave_publica_x25519(&sk_desktop);
    let (client_hello, client_keys) = construir_client_hello(&sk_android, &pk_desktop, &psk_token).unwrap();
    let (_, server_keys) = responder_server_hello(&sk_desktop, &psk_token, &client_hello).unwrap();

    let mut cipher_android = SessionCipher::para_cliente(&client_keys);
    let mut cipher_desktop = SessionCipher::para_servidor(&server_keys);

    // 1. Android -> Desktop
    let mensaje_a2d = b"SYNC_PULL_REQUEST: revision=42, client=Android-1.14.9";
    let trama_1 = cipher_android.seal(mensaje_a2d).unwrap();
    assert_eq!(trama_1.len(), 8 + mensaje_a2d.len() + 16);
    assert_eq!(cipher_android.tx_seq(), 1);

    let descifrado_1 = cipher_desktop.open(&trama_1).unwrap();
    assert_eq!(descifrado_1, mensaje_a2d);
    assert_eq!(cipher_desktop.rx_seq(), 1);

    // 2. Desktop -> Android
    let mensaje_d2a = b"SYNC_DIFF_PAYLOAD: 12 entries updated, revision=43";
    let trama_2 = cipher_desktop.seal(mensaje_d2a).unwrap();
    assert_eq!(cipher_desktop.tx_seq(), 1);

    let descifrado_2 = cipher_android.open(&trama_2).unwrap();
    assert_eq!(descifrado_2, mensaje_d2a);
    assert_eq!(cipher_android.rx_seq(), 1);

    // 3. Segundo mensaje consecutivo Android -> Desktop (comprobando secuencia monotónica)
    let mensaje_a2d_2 = b"SYNC_ACK: revision=43 received";
    let trama_3 = cipher_android.seal(mensaje_a2d_2).unwrap();
    assert_eq!(cipher_android.tx_seq(), 2);

    let descifrado_3 = cipher_desktop.open(&trama_3).unwrap();
    assert_eq!(descifrado_3, mensaje_a2d_2);
    assert_eq!(cipher_desktop.rx_seq(), 2);
}

#[test]
fn test_rechazo_estricto_ataque_replay_y_desorden() {
    let sk_desktop: [u8; 32] = [0x11; 32];
    let sk_android: [u8; 32] = [0x22; 32];
    let psk_token: [u8; 32] = [0x33; 32];

    let pk_desktop = derivar_clave_publica_x25519(&sk_desktop);
    let (client_hello, client_keys) = construir_client_hello(&sk_android, &pk_desktop, &psk_token).unwrap();
    let (_, server_keys) = responder_server_hello(&sk_desktop, &psk_token, &client_hello).unwrap();

    let mut cipher_android = SessionCipher::para_cliente(&client_keys);
    let mut cipher_desktop = SessionCipher::para_servidor(&server_keys);

    let trama_0 = cipher_android.seal(b"Mensaje 0").unwrap();
    let trama_1 = cipher_android.seal(b"Mensaje 1").unwrap();

    // Replay ataque: enviar trama_0 de nuevo tras haberla consumido
    let ok_0 = cipher_desktop.open(&trama_0);
    assert!(ok_0.is_ok());

    let replay_0 = cipher_desktop.open(&trama_0);
    assert!(
        replay_0.is_err(),
        "El servidor debe rechazar categóricamente un paquete ya procesado (Replay)"
    );

    // Desorden: procesar trama_1 ahora sí debe tener éxito
    let ok_1 = cipher_desktop.open(&trama_1);
    assert!(ok_1.is_ok());
}

#[test]
fn test_rechazo_trama_corrupta_poly1305() {
    let sk_desktop: [u8; 32] = [0x11; 32];
    let sk_android: [u8; 32] = [0x22; 32];
    let psk_token: [u8; 32] = [0x33; 32];

    let pk_desktop = derivar_clave_publica_x25519(&sk_desktop);
    let (client_hello, client_keys) = construir_client_hello(&sk_android, &pk_desktop, &psk_token).unwrap();
    let (_, server_keys) = responder_server_hello(&sk_desktop, &psk_token, &client_hello).unwrap();

    let mut cipher_android = SessionCipher::para_cliente(&client_keys);
    let mut cipher_desktop = SessionCipher::para_servidor(&server_keys);

    let mut trama = cipher_android.seal(b"Dato secreto").unwrap();

    // Corromper el último byte (parte de la etiqueta Poly1305)
    let len = trama.len();
    trama[len - 1] ^= 0x01;

    let res = cipher_desktop.open(&trama);
    assert!(
        res.is_err(),
        "ChaCha20Poly1305 debe fallar inmediatamente si el ciphertext o tag se alteran"
    );
}

#[test]
fn test_rechazo_trama_demasiado_corta() {
    let mut cipher = SessionCipher::new([0xaa; 32], [0xbb; 32]);
    let corta = [0u8; 23];
    assert!(cipher.open(&corta).is_err());
}
