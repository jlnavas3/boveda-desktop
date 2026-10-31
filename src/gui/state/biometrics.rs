/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Integración con el llavero seguro del sistema operativo (Secret Service / Windows Credential Manager)
//! para permitir desbloqueo biométrico y rápido sin fricción.

use keyring::Entry;

const SERVICE_NAME: &str = "boveda-local-desktop";

fn normalizar_usuario(ruta_boveda: &str) -> String {
    use sha1::Digest;
    let mut hasher = sha1::Sha1::new();
    hasher.update(ruta_boveda.as_bytes());
    let res = hasher.finalize();
    let mut s = String::with_capacity(40);
    for b in res {
        use std::fmt::Write;
        let _ = write!(s, "{:02x}", b);
    }
    format!("vault_{}", &s[..16])
}

/// Guarda la contraseña en el llavero protegido del sistema operativo.
pub fn guardar_clave_biometria(ruta_boveda: &str, clave: &str) -> Result<(), String> {
    let usuario = normalizar_usuario(ruta_boveda);
    let entry = Entry::new(SERVICE_NAME, &usuario)
        .map_err(|e| format!("Error accediendo al llavero seguro del sistema: {e}"))?;
    entry
        .set_password(clave)
        .map_err(|e| format!("Error guardando clave en el llavero del SO: {e}"))?;
    Ok(())
}

/// Recupera la contraseña del llavero protegido del sistema operativo.
pub fn obtener_clave_biometria(ruta_boveda: &str) -> Result<String, String> {
    let usuario = normalizar_usuario(ruta_boveda);
    let entry = Entry::new(SERVICE_NAME, &usuario)
        .map_err(|e| format!("Error accediendo al llavero seguro del sistema: {e}"))?;
    entry
        .get_password()
        .map_err(|e| format!("No se encontró clave guardada para esta bóveda: {e}"))
}

/// Elimina la contraseña del llavero seguro del sistema operativo.
pub fn eliminar_clave_biometria(ruta_boveda: &str) -> Result<(), String> {
    let usuario = normalizar_usuario(ruta_boveda);
    let entry = Entry::new(SERVICE_NAME, &usuario)
        .map_err(|e| format!("Error accediendo al llavero seguro del sistema: {e}"))?;
    let _ = entry.delete_password();
    Ok(())
}

/// Comprueba si existe una credencial guardada para la bóveda dada.
pub fn esta_biometria_disponible(ruta_boveda: &str) -> bool {
    let usuario = normalizar_usuario(ruta_boveda);
    if let Ok(entry) = Entry::new(SERVICE_NAME, &usuario) {
        entry.get_password().is_ok()
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalizar_usuario_determinista() {
        let u1 = normalizar_usuario("/home/user/vault.bvda");
        let u2 = normalizar_usuario("/home/user/vault.bvda");
        assert_eq!(u1, u2);
        assert!(u1.starts_with("vault_"));
    }
}
