/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

use crate::models::Entrada;
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[inline]
fn escribir_campo(hasher: &mut Sha256, texto: &str) {
    let bytes = texto.as_bytes();
    let len = bytes.len() as u32;
    hasher.update(len.to_le_bytes());
    hasher.update(bytes);
}

/// Calcula el hash canónico de 32 bytes del contenido de una entrada.
/// Excluye estrictamente los metadatos de sincronización (`revision` y `last_sync_hash`).
pub fn calcular_content_hash(e: &Entrada) -> [u8; 32] {
    let mut hasher = Sha256::new();

    // 1. UUID binario (16 bytes)
    let uuid_bytes = Uuid::parse_str(&e.id)
        .map(|u| *u.as_bytes())
        .unwrap_or([0u8; 16]);
    hasher.update(uuid_bytes);

    // 2. Campos nucleares en orden canónico
    escribir_campo(&mut hasher, &e.titulo);
    escribir_campo(&mut hasher, &e.usuario);
    escribir_campo(&mut hasher, &e.contrasena);
    escribir_campo(&mut hasher, &e.notas);
    escribir_campo(&mut hasher, e.secreto_totp.as_deref().unwrap_or(""));

    // 3. URLs ordenadas
    let len_urls = e.urls.len() as u32;
    hasher.update(len_urls.to_le_bytes());
    for url in &e.urls {
        escribir_campo(&mut hasher, url);
    }

    // 4. Estado de eliminación
    let eliminado: u8 = if e.eliminada_en > 0 { 1 } else { 0 };
    hasher.update([eliminado]);

    hasher.finalize().into()
}
