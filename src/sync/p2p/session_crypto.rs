/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Túnel simétrico autenticado ChaCha20-Poly1305 con protección estricta Anti-Replay.
//! Contrato: Nonce de 12 bytes ([0u8; 4] || seq_le) y AAD de 8 bytes (seq_le).

use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    ChaCha20Poly1305, Key, Nonce,
};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::sync::p2p::handshake::HandshakeKeys;

/// Cifrador de sesión simétrica con control monotónico estricto de números de secuencia.
/// Limpia automáticamente las claves criptográficas de la RAM en su `Drop`.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SessionCipher {
    tx_key: [u8; 32],
    rx_key: [u8; 32],
    #[zeroize(skip)]
    tx_seq: u64,
    #[zeroize(skip)]
    rx_seq: u64,
}

impl SessionCipher {
    /// Crea un nuevo cifrador de sesión con claves de transmisión y recepción independientes.
    pub fn new(tx_key: [u8; 32], rx_key: [u8; 32]) -> Self {
        Self {
            tx_key,
            rx_key,
            tx_seq: 0,
            rx_seq: 0,
        }
    }

    /// Inicializa la sesión para el rol Servidor (Desktop):
    /// - Transmite (TX) usando `key_d2a` (Desktop -> Android).
    /// - Recibe (RX) usando `key_a2d` (Android -> Desktop).
    pub fn para_servidor(keys: &HandshakeKeys) -> Self {
        Self::new(keys.key_d2a, keys.key_a2d)
    }

    /// Inicializa la sesión para el rol Cliente (Android):
    /// - Transmite (TX) usando `key_a2d` (Android -> Desktop).
    /// - Recibe (RX) usando `key_d2a` (Desktop -> Android).
    pub fn para_cliente(keys: &HandshakeKeys) -> Self {
        Self::new(keys.key_a2d, keys.key_d2a)
    }

    /// Número de secuencia del siguiente mensaje a transmitir.
    pub fn tx_seq(&self) -> u64 {
        self.tx_seq
    }

    /// Número de secuencia esperado para el siguiente mensaje a recibir.
    pub fn rx_seq(&self) -> u64 {
        self.rx_seq
    }

    /// Cifra un mensaje en claro (`plaintext`) produciendo una trama autenticada:
    /// - `Nonce (12B)` = `[0u8; 4] || tx_seq (8B en little-endian)`
    /// - `AAD (8B)` = `tx_seq (8B en little-endian)`
    /// - Cifra con ChaCha20-Poly1305 generando texto cifrado + tag (16B).
    /// - Incrementa monótonamente `tx_seq += 1`.
    /// - Retorna `seq_le (8B) || ciphertext_with_tag`.
    pub fn seal(&mut self, plaintext: &[u8]) -> Result<Vec<u8>, String> {
        let current_seq = self.tx_seq;
        let seq_bytes = current_seq.to_le_bytes();

        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[4..12].copy_from_slice(&seq_bytes);

        let cipher = ChaCha20Poly1305::new(Key::from_slice(&self.tx_key));
        let payload = Payload {
            msg: plaintext,
            aad: &seq_bytes,
        };

        let ciphertext_with_tag = cipher
            .encrypt(Nonce::from_slice(&nonce_bytes), payload)
            .map_err(|e| format!("Error cifrando trama P2P: {e}"))?;

        self.tx_seq = self
            .tx_seq
            .checked_add(1)
            .ok_or_else(|| "Desbordamiento monotónico del número de secuencia tx_seq".to_string())?;

        let mut frame = Vec::with_capacity(8 + ciphertext_with_tag.len());
        frame.extend_from_slice(&seq_bytes);
        frame.extend_from_slice(&ciphertext_with_tag);

        Ok(frame)
    }

    /// Descifra y valida la autenticidad e integridad de una trama P2P recibida:
    /// - Requiere un tamaño mínimo de 24 bytes (8B secuencia + 16B tag Poly1305).
    /// - Extrae el número de secuencia little-endian de los primeros 8 bytes.
    /// - Exige estrictamente `seq == self.rx_seq` para neutralizar ataques de repetición o desorden.
    /// - Descifra con ChaCha20-Poly1305 usando Nonce `[0u8; 4] || seq_le` y AAD `seq_le`.
    /// - Incrementa monótonamente `rx_seq += 1`.
    /// - Retorna el vector con el texto en claro.
    pub fn open(&mut self, frame: &[u8]) -> Result<Vec<u8>, String> {
        if frame.len() < 24 {
            return Err(format!(
                "Trama P2P demasiado corta: longitud {} bytes (mínimo 24)",
                frame.len()
            ));
        }

        let mut seq_bytes = [0u8; 8];
        seq_bytes.copy_from_slice(&frame[0..8]);
        let seq = u64::from_le_bytes(seq_bytes);

        if seq != self.rx_seq {
            return Err(format!(
                "Ataque Replay o trama fuera de secuencia detectada: esperado seq={}, recibido seq={}",
                self.rx_seq, seq
            ));
        }

        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[4..12].copy_from_slice(&seq_bytes);

        let cipher = ChaCha20Poly1305::new(Key::from_slice(&self.rx_key));
        let payload = Payload {
            msg: &frame[8..],
            aad: &seq_bytes,
        };

        let plaintext = cipher
            .decrypt(Nonce::from_slice(&nonce_bytes), payload)
            .map_err(|e| format!("Fallo de integridad o autenticidad en trama P2P: {e}"))?;

        self.rx_seq = self
            .rx_seq
            .checked_add(1)
            .ok_or_else(|| "Desbordamiento monotónico del número de secuencia rx_seq".to_string())?;

        Ok(plaintext)
    }
}
