/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

use sha2::{Digest, Sha256};

/// Calcula la huella visual determinista de 6 bytes en formato "XX:XX:XX:XX:XX:XX"
/// a partir de SHA-256(clave_publica || token_psk).
pub fn calcular_huella_visual(clave_publica: &[u8; 32], token_psk: &[u8; 32]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(clave_publica);
    hasher.update(token_psk);
    let digest = hasher.finalize();

    let mut res = String::with_capacity(17);
    for (i, byte) in digest[..6].iter().enumerate() {
        use std::fmt::Write;
        let _ = write!(res, "{:02X}", byte);
        if i < 5 {
            res.push(':');
        }
    }
    res
}
