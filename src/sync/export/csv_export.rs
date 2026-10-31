/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Exportador a formato CSV estándar compatible con RFC 4180.

use crate::models::Entrada;

/// Exporta una lista de entradas a formato CSV plano.
pub fn exportar_csv(entradas: &[Entrada]) -> String {
    let mut salida = String::new();
    salida.push_str("id,type,title,username,password,urls,notes,totp,favorite\n");

    for e in entradas {
        let id_esc = escapar_campo_csv(&e.id);
        let tipo_esc = escapar_campo_csv(e.tipo.etiqueta());
        let titulo_esc = escapar_campo_csv(&e.titulo);
        let usuario_esc = escapar_campo_csv(&e.usuario);
        let contrasena_esc = escapar_campo_csv(&e.contrasena);
        let urls_esc = escapar_campo_csv(&e.urls.join(";"));
        let notas_esc = escapar_campo_csv(&e.notas);
        let totp_esc = escapar_campo_csv(e.secreto_totp.as_deref().unwrap_or(""));
        let favorito = if e.favorito { "true" } else { "false" };

        salida.push_str(&format!(
            "{},{},{},{},{},{},{},{},{}\n",
            id_esc, tipo_esc, titulo_esc, usuario_esc, contrasena_esc, urls_esc, notas_esc, totp_esc, favorito
        ));
    }

    salida
}

fn escapar_campo_csv(campo: &str) -> String {
    if campo.contains(',') || campo.contains('"') || campo.contains('\n') || campo.contains('\r') {
        let escapado = campo.replace('"', "\"\"");
        format!("\"{}\"", escapado)
    } else {
        campo.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TipoEntrada;

    #[test]
    fn test_exportar_csv() {
        let entradas = vec![
            Entrada {
                id: "1".into(),
                titulo: "Google, Inc.".into(), // Contiene coma
                usuario: "user@gmail.com".into(),
                contrasena: "pass\"word".into(), // Contiene comillas
                urls: vec!["https://google.com".into()],
                notas: "Nota con\nsalto".into(),
                tipo: TipoEntrada::Login,
                favorito: true,
                ..Default::default()
            }
        ];

        let csv = exportar_csv(&entradas);
        assert!(csv.contains("\"Google, Inc.\""));
        assert!(csv.contains("\"pass\"\"word\""));
        assert!(csv.contains("\"Nota con\nsalto\""));
    }
}
