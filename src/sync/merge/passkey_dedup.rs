/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Deduplicación y consolidación de llaves de acceso (Passkeys FIDO2 / WebAuthn).
//! Réplica exacta de la lógica de FusionadorEntradas.kt de Android.

use crate::models::passkey::DatosPasskey;

/// Determina si dos llaves de acceso representan criptográfica y técnicamente la misma credencial.
pub fn son_misma_passkey(p1: Option<&DatosPasskey>, p2: Option<&DatosPasskey>) -> bool {
    let (Some(p1), Some(p2)) = (p1, p2) else {
        return false;
    };

    // 1. Mismo ID de credencial WebAuthn (Base64Url)
    if !p1.cred_id.trim().is_empty()
        && !p2.cred_id.trim().is_empty()
        && p1.cred_id.trim().eq_ignore_ascii_case(p2.cred_id.trim())
    {
        return true;
    }

    // 2. Misma clave privada
    if !p1.clave_privada.trim().is_empty()
        && !p2.clave_privada.trim().is_empty()
        && p1.clave_privada.trim() == p2.clave_privada.trim()
    {
        return true;
    }

    // 3. Mismo rpId y userHandle no vacío
    if !p1.rp_id.trim().is_empty()
        && !p2.rp_id.trim().is_empty()
        && p1.rp_id.trim().eq_ignore_ascii_case(p2.rp_id.trim())
        && !p1.user_handle.trim().is_empty()
        && !p2.user_handle.trim().is_empty()
        && p1.user_handle.trim() == p2.user_handle.trim()
    {
        return true;
    }

    false
}

/// Consolida los metadatos más completos de dos instancias de la misma Passkey.
pub fn consolidar_passkey(base: &DatosPasskey, otra: &DatosPasskey) -> DatosPasskey {
    let usuario = if !base.usuario.trim().is_empty() {
        base.usuario.clone()
    } else {
        otra.usuario.clone()
    };

    let rp_name = if !base.rp_name.trim().is_empty()
        && !base.rp_name.trim().eq_ignore_ascii_case(base.rp_id.trim())
    {
        base.rp_name.clone()
    } else if !otra.rp_name.trim().is_empty() {
        otra.rp_name.clone()
    } else {
        base.rp_name.clone()
    };

    let user_handle = if !base.user_handle.trim().is_empty() {
        base.user_handle.clone()
    } else {
        otra.user_handle.clone()
    };

    let creada_en = if base.creada_en > 0 {
        base.creada_en
    } else {
        otra.creada_en
    };

    DatosPasskey {
        rp_id: base.rp_id.clone(),
        rp_name,
        user_handle,
        cred_id: base.cred_id.clone(),
        clave_privada: base.clave_privada.clone(),
        algoritmo: base.algoritmo.clone(),
        usuario,
        creada_en,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_son_misma_passkey_por_cred_id() {
        let p1 = DatosPasskey {
            rp_id: "google.com".into(),
            rp_name: "Google".into(),
            user_handle: "h1".into(),
            cred_id: "CRED123".into(),
            clave_privada: "priv1".into(),
            algoritmo: "ES256".into(),
            usuario: "user1".into(),
            creada_en: 100,
        };
        let p2 = DatosPasskey {
            rp_id: "google.com".into(),
            rp_name: "".into(),
            user_handle: "h1".into(),
            cred_id: "cred123".into(), // case-insensitive
            clave_privada: "priv2".into(),
            algoritmo: "ES256".into(),
            usuario: "".into(),
            creada_en: 200,
        };

        assert!(son_misma_passkey(Some(&p1), Some(&p2)));
    }

    #[test]
    fn test_consolidar_passkey() {
        let base = DatosPasskey {
            rp_id: "github.com".into(),
            rp_name: "github.com".into(), // Es igual al rp_id
            user_handle: "".into(),
            cred_id: "ID1".into(),
            clave_privada: "KEY1".into(),
            algoritmo: "ES256".into(),
            usuario: "".into(),
            creada_en: 0,
        };
        let otra = DatosPasskey {
            rp_id: "github.com".into(),
            rp_name: "GitHub Official".into(),
            user_handle: "handle_git".into(),
            cred_id: "ID1".into(),
            clave_privada: "KEY1".into(),
            algoritmo: "ES256".into(),
            usuario: "dev@git.com".into(),
            creada_en: 1700000000000,
        };

        let cons = consolidar_passkey(&base, &otra);
        assert_eq!(cons.rp_name, "GitHub Official");
        assert_eq!(cons.user_handle, "handle_git");
        assert_eq!(cons.usuario, "dev@git.com");
        assert_eq!(cons.creada_en, 1700000000000);
    }
}
