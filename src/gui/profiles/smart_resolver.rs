/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Resolución inteligente de perfiles/identidades asociada a credenciales.
//! Réplica fiel en Rust de `ResolverIdentidadEntrada.kt` de Android.

use crate::models::{Entrada, Perfil};

/// Resuelve el perfil asociado a una credencial:
/// Prioridad 1: Asignación explícita mediante `entrada.perfil_id`.
/// Prioridad 2: Vinculación inteligente automática comparando el nombre de usuario
/// con el correo principal, alias secundarios o número de documento de identidad del perfil.
pub fn resolver_perfil_para_entrada<'a>(
    entrada: &Entrada,
    perfiles: &'a [Perfil],
) -> Option<&'a Perfil> {
    if perfiles.is_empty() {
        return None;
    }

    // Prioridad 1: Asignación explícita
    if let Some(ref explicit_id) = entrada.perfil_id {
        if let Some(p) = perfiles.iter().find(|p| p.id == *explicit_id) {
            return Some(p);
        }
    }

    let usuario_norm = entrada.usuario.trim().to_lowercase();
    if usuario_norm.is_empty() {
        return None;
    }

    // Prioridad 2: Vinculación inteligente
    perfiles.iter().find(|p| {
        let coincide_correo_principal = !p.correo_principal.is_empty()
            && p.correo_principal.trim().to_lowercase() == usuario_norm;

        let coincide_alias = p
            .correos_secundarios
            .iter()
            .any(|alias| alias.trim().to_lowercase() == usuario_norm);

        let coincide_documento = match &p.numero_documento {
            Some(doc) if !doc.trim().is_empty() => es_mismo_documento(doc, &usuario_norm),
            _ => false,
        };

        coincide_correo_principal || coincide_alias || coincide_documento
    })
}

/// Compara dos cadenas de documento ignorando espacios, guiones o caracteres especiales.
pub fn es_mismo_documento(doc_identidad: &str, doc_usuario: &str) -> bool {
    let norm1 = doc_identidad.trim().to_lowercase();
    let norm2 = doc_usuario.trim().to_lowercase();
    if norm1 == norm2 {
        return true;
    }
    let limpio1: String = norm1.chars().filter(|c| c.is_alphanumeric()).collect();
    let limpio2: String = norm2.chars().filter(|c| c.is_alphanumeric()).collect();
    !limpio1.is_empty() && limpio1 == limpio2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vinculacion_inteligente_perfil() {
        let perfiles = vec![Perfil {
            id: "perfil-personal".to_string(),
            nombre: "Personal".to_string(),
            correo_principal: "jolunavi@gmail.com".to_string(),
            correos_secundarios: vec!["jlnavas@empresa.com".to_string()],
            numero_documento: Some("1204620338".to_string()),
            ..Default::default()
        }];

        // Coincidencia por correo principal
        let e1 = Entrada {
            usuario: "jolunavi@gmail.com".to_string(),
            ..Default::default()
        };
        assert_eq!(
            resolver_perfil_para_entrada(&e1, &perfiles).map(|p| p.id.as_str()),
            Some("perfil-personal")
        );

        // Coincidencia por alias
        let e2 = Entrada {
            usuario: "jlnavas@empresa.com".to_string(),
            ..Default::default()
        };
        assert_eq!(
            resolver_perfil_para_entrada(&e2, &perfiles).map(|p| p.id.as_str()),
            Some("perfil-personal")
        );

        // Coincidencia por documento
        let e3 = Entrada {
            usuario: "1204620338".to_string(),
            ..Default::default()
        };
        assert_eq!(
            resolver_perfil_para_entrada(&e3, &perfiles).map(|p| p.id.as_str()),
            Some("perfil-personal")
        );

        // Sin coincidencia
        let e4 = Entrada {
            usuario: "desconocido@otro.com".to_string(),
            ..Default::default()
        };
        assert_eq!(resolver_perfil_para_entrada(&e4, &perfiles), None);
    }
}
