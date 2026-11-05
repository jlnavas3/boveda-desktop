/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Analizador desacoplado de discrepancias previo a la unificación de entradas (paridad con `AnalizadorDiscrepanciasUnificacion` de Android).

use std::collections::HashSet;

use crate::gui::service_grouper::extraer_nombre_servicio;
use crate::models::Entrada;

/// Genera un texto explicativo de advertencias y respaldo sin pérdida para mostrar en el diálogo de unificación.
pub fn analizar_discrepancias_unificacion(entradas: &[Entrada], principal_id: &str) -> String {
    if entradas.len() < 2 {
        return String::new();
    }

    let mut servicios = Vec::new();
    let mut usuarios = Vec::new();
    let mut claves = HashSet::new();
    let mut totps = HashSet::new();
    let mut passkeys = HashSet::new();

    for e in entradas {
        let u0 = e.urls.first().map(|s| s.as_str()).unwrap_or("");
        let serv = extraer_nombre_servicio(u0, &e.titulo).to_lowercase();
        if !serv.is_empty() && !servicios.contains(&serv) {
            servicios.push(serv);
        }
        let usr = e.usuario.trim().to_string();
        if !usr.is_empty() && !usuarios.contains(&usr) {
            usuarios.push(usr);
        }
        if !e.contrasena.trim().is_empty() {
            claves.insert(e.contrasena.clone());
        }
        if let Some(ref t) = e.secreto_totp {
            let clean = t.replace(' ', "").trim().to_uppercase();
            if !clean.is_empty() {
                totps.insert(clean);
            }
        }
        for ts in &e.totps_secundarios {
            let clean = ts.secreto.replace(' ', "").trim().to_uppercase();
            if !clean.is_empty() {
                totps.insert(clean);
            }
        }
        if let Some(ref pk) = e.passkey {
            let k = if !pk.cred_id.is_empty() {
                pk.cred_id.clone()
            } else {
                format!("{}:{}", pk.rp_id, pk.usuario)
            };
            passkeys.insert(k);
        }
    }

    let principal_titulo = entradas
        .iter()
        .find(|e| e.id == principal_id)
        .map(|e| e.titulo.as_str())
        .unwrap_or("Entrada principal");

    let mut lineas = vec![format!(
        "• Registro principal seleccionado: \"{principal_titulo}\" (conservará su título y datos prioritarios)."
    )];

    if servicios.len() > 1 {
        lineas.push(format!(
            "⚠ Atención: Se detectaron {} servicios distintos ({}).",
            servicios.len(),
            servicios.join(", ")
        ));
    }
    if usuarios.len() > 1 {
        lineas.push(format!(
            "• Usuarios distintos detectados ({}): los alternativos se respaldarán en las notas.",
            usuarios.join(", ")
        ));
    }
    if claves.len() > 1 {
        lineas.push(format!(
            "• {} contraseñas distintas: las secundarias se guardarán en 'Credenciales alternativas → Contraseñas anteriores'.",
            claves.len()
        ));
    }
    if totps.len() > 1 {
        lineas.push(format!(
            "• {} códigos 2FA distintos: los adicionales se conservarán en 'Credenciales alternativas → Códigos 2FA'.",
            totps.len()
        ));
    }
    if passkeys.len() > 1 {
        lineas.push(format!(
            "• {} llaves de acceso (Passkeys) distintas: las adicionales se conservarán en 'Credenciales alternativas'.",
            passkeys.len()
        ));
    }
    lineas.push("• Ningún dato se perderá: URLs, etiquetas, categorías y campos personalizados se combinarán automáticamente.".to_string());

    lineas.join("\n")
}
