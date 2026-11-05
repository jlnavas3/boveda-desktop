/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Handshake 1-RTT con autenticación mutua basada en PSK y X25519 para sincronización P2P.
//! Contrato criptográfico canónico: X25519 + HKDF-SHA256 + HMAC-SHA256.

use hkdf::Hkdf;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use x25519_dalek::{PublicKey, StaticSecret};
use zeroize::{Zeroize, ZeroizeOnDrop};

type HmacSha256 = Hmac<Sha256>;

const HKDF_INFO_PREFIX: &[u8] = b"BVDA-P2P-V2";
const CLIENT_AUTH_LABEL: &[u8] = b"CLIENT-AUTH";
const SERVER_AUTH_LABEL: &[u8] = b"SERVER-AUTH";

/// Conjunto de claves de sesión derivadas de forma simétrica e inmutable.
/// Limpia automáticamente de la memoria RAM todas las claves secretas al destruirse.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct HandshakeKeys {
    /// Clave AEAD para tramas Android -> Desktop (32 bytes).
    pub key_a2d: [u8; 32],
    /// Clave AEAD para tramas Desktop -> Android (32 bytes).
    pub key_d2a: [u8; 32],
    /// Clave HMAC de autenticación mutua del Handshake (32 bytes).
    pub handshake_key: [u8; 32],
}

impl HandshakeKeys {
    pub fn new(key_a2d: [u8; 32], key_d2a: [u8; 32], handshake_key: [u8; 32]) -> Self {
        Self {
            key_a2d,
            key_d2a,
            handshake_key,
        }
    }
}

/// Deriva la clave pública X25519 (32 bytes) a partir de una clave privada escalar (32 bytes).
pub fn derivar_clave_publica_x25519(sk: &[u8; 32]) -> [u8; 32] {
    let secret = StaticSecret::from(*sk);
    let public = PublicKey::from(&secret);
    *public.as_bytes()
}

/// Calcula el secreto compartido Diffie-Hellman mediante X25519 (32 bytes).
pub fn calcular_diffie_hellman(sk_propia: &[u8; 32], pk_remota: &[u8; 32]) -> [u8; 32] {
    let secret = StaticSecret::from(*sk_propia);
    let public = PublicKey::from(*pk_remota);
    let shared = secret.diffie_hellman(&public);
    *shared.as_bytes()
}

/// Deriva las 3 claves de sesión usando HKDF-SHA256:
/// - salt: `psk_token` (32 bytes)
/// - ikm: `dh_shared` (32 bytes)
/// - info: `b"BVDA-P2P-V2" || pk_desktop || pk_android` (75 bytes)
///
/// Retorna `(key_a2d, key_d2a, handshake_key)` expandidas a 96 bytes.
pub fn derivar_claves_sesion(
    dh_shared: &[u8; 32],
    psk_token: &[u8; 32],
    pk_desktop: &[u8; 32],
    pk_android: &[u8; 32],
) -> Result<HandshakeKeys, String> {
    let mut info = Vec::with_capacity(HKDF_INFO_PREFIX.len() + 32 + 32);
    info.extend_from_slice(HKDF_INFO_PREFIX);
    info.extend_from_slice(pk_desktop);
    info.extend_from_slice(pk_android);

    let hk = Hkdf::<Sha256>::new(Some(psk_token), dh_shared);
    let mut okm = [0u8; 96];
    hk.expand(&info, &mut okm)
        .map_err(|e| format!("Fallo al expandir HKDF-SHA256: {e}"))?;

    let mut key_a2d = [0u8; 32];
    let mut key_d2a = [0u8; 32];
    let mut handshake_key = [0u8; 32];

    key_a2d.copy_from_slice(&okm[0..32]);
    key_d2a.copy_from_slice(&okm[32..64]);
    handshake_key.copy_from_slice(&okm[64..96]);

    okm.zeroize();

    Ok(HandshakeKeys::new(key_a2d, key_d2a, handshake_key))
}

/// Genera la trama `ClientHello` (64 bytes, Android -> Desktop):
/// `pk_android (32B) || HMAC-SHA256(handshake_key, b"CLIENT-AUTH" || pk_android || pk_desktop) (32B)`.
pub fn construir_client_hello(
    sk_android: &[u8; 32],
    pk_desktop: &[u8; 32],
    psk_token: &[u8; 32],
) -> Result<([u8; 64], HandshakeKeys), String> {
    let pk_android = derivar_clave_publica_x25519(sk_android);
    let dh_shared = calcular_diffie_hellman(sk_android, pk_desktop);
    let keys = derivar_claves_sesion(&dh_shared, psk_token, pk_desktop, &pk_android)?;

    let mut mac_data = Vec::with_capacity(CLIENT_AUTH_LABEL.len() + 32 + 32);
    mac_data.extend_from_slice(CLIENT_AUTH_LABEL);
    mac_data.extend_from_slice(&pk_android);
    mac_data.extend_from_slice(pk_desktop);

    let mut mac = HmacSha256::new_from_slice(&keys.handshake_key)
        .map_err(|e| format!("Clave HMAC inválida: {e}"))?;
    mac.update(&mac_data);
    let auth_tag = mac.finalize().into_bytes();

    let mut client_hello = [0u8; 64];
    client_hello[0..32].copy_from_slice(&pk_android);
    client_hello[32..64].copy_from_slice(&auth_tag);

    Ok((client_hello, keys))
}

/// Procesa la trama `ClientHello` en el Desktop (servidor) y genera la respuesta `ServerHello`:
/// 1. Extrae `pk_android` (primeros 32 bytes de `client_hello`).
/// 2. Calcula `dh_shared = X25519(sk_desktop, pk_android)`.
/// 3. Deriva las claves de sesión mediante HKDF-SHA256.
/// 4. Valida en tiempo constante `client_hello[32..64] == HMAC(handshake_key, b"CLIENT-AUTH" || pk_android || pk_desktop)`.
/// 5. Genera `ServerHello` (32 bytes): `HMAC(handshake_key, b"SERVER-AUTH" || pk_desktop || pk_android)`.
pub fn responder_server_hello(
    sk_desktop: &[u8; 32],
    psk_token: &[u8; 32],
    client_hello: &[u8; 64],
) -> Result<([u8; 32], HandshakeKeys), String> {
    let pk_desktop = derivar_clave_publica_x25519(sk_desktop);

    let mut pk_android = [0u8; 32];
    pk_android.copy_from_slice(&client_hello[0..32]);

    let dh_shared = calcular_diffie_hellman(sk_desktop, &pk_android);
    let keys = derivar_claves_sesion(&dh_shared, psk_token, &pk_desktop, &pk_android)?;

    // Verificar autenticación del cliente
    let mut mac_data = Vec::with_capacity(CLIENT_AUTH_LABEL.len() + 32 + 32);
    mac_data.extend_from_slice(CLIENT_AUTH_LABEL);
    mac_data.extend_from_slice(&pk_android);
    mac_data.extend_from_slice(&pk_desktop);

    let mut mac_client = HmacSha256::new_from_slice(&keys.handshake_key)
        .map_err(|e| format!("Clave HMAC inválida: {e}"))?;
    mac_client.update(&mac_data);
    mac_client
        .verify_slice(&client_hello[32..64])
        .map_err(|_| "Autenticación de ClientHello fallida: token PSK o clave pública discordante".to_string())?;

    // Generar ServerHello
    let mut server_data = Vec::with_capacity(SERVER_AUTH_LABEL.len() + 32 + 32);
    server_data.extend_from_slice(SERVER_AUTH_LABEL);
    server_data.extend_from_slice(&pk_desktop);
    server_data.extend_from_slice(&pk_android);

    let mut mac_server = HmacSha256::new_from_slice(&keys.handshake_key)
        .map_err(|e| format!("Clave HMAC inválida: {e}"))?;
    mac_server.update(&server_data);
    let server_tag = mac_server.finalize().into_bytes();

    let mut server_hello = [0u8; 32];
    server_hello.copy_from_slice(&server_tag);

    Ok((server_hello, keys))
}

/// Valida la trama `ServerHello` (32 bytes) en el cliente (Android):
/// Verifica `server_hello == HMAC-SHA256(handshake_key, b"SERVER-AUTH" || pk_desktop || pk_android)`.
pub fn verificar_server_hello(
    handshake_key: &[u8; 32],
    pk_desktop: &[u8; 32],
    pk_android: &[u8; 32],
    server_hello: &[u8; 32],
) -> Result<(), String> {
    let mut server_data = Vec::with_capacity(SERVER_AUTH_LABEL.len() + 32 + 32);
    server_data.extend_from_slice(SERVER_AUTH_LABEL);
    server_data.extend_from_slice(pk_desktop);
    server_data.extend_from_slice(pk_android);

    let mut mac = HmacSha256::new_from_slice(handshake_key)
        .map_err(|e| format!("Clave HMAC inválida: {e}"))?;
    mac.update(&server_data);
    mac.verify_slice(server_hello)
        .map_err(|_| "Autenticación de ServerHello fallida: Desktop no autenticado con PSK".to_string())
}
