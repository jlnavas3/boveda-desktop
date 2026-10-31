/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Importador universal de archivos CSV (Google Passwords, Chrome, KeePass, LastPass, etc.).
//! Compatible con la heurística y detección de ImportadorCsv.kt en Android.

use crate::crypto::BvdaError;
use crate::models::{Entrada, TipoEntrada};
use chrono::Utc;

const ALIAS_TITULO: &[&str] = &["name", "title", "nombre", "item name", "label"];
const ALIAS_URL: &[&str] = &["url", "urls", "login_uri", "uri", "website", "site", "sitio"];
const ALIAS_USUARIO: &[&str] = &["username", "login_username", "usuario", "user", "email"];
const ALIAS_CONTRASENA: &[&str] = &["password", "login_password", "contraseña", "contrasena", "clave"];
const ALIAS_NOTAS: &[&str] = &["notes", "note", "extra", "notas", "comentario", "comentarios"];
const ALIAS_TOTP: &[&str] = &["totp", "otp", "2fa", "one-time-code"];

/// Detecta automáticamente el delimitador más probable (',', ';', '\t', '|').
pub fn detectar_delimitador(texto: &str) -> char {
    let primera_linea = match texto.lines().find(|l| !l.trim().is_empty()) {
        Some(l) => l,
        None => return ',',
    };

    let mut en_comillas = false;
    let mut comas = 0;
    let mut puntos_y_comas = 0;
    let mut tabuladores = 0;
    let mut plecas = 0;

    for c in primera_linea.chars() {
        if c == '"' {
            en_comillas = !en_comillas;
        } else if !en_comillas {
            match c {
                ',' => comas += 1,
                ';' => puntos_y_comas += 1,
                '\t' => tabuladores += 1,
                '|' => plecas += 1,
                _ => {}
            }
        }
    }

    let max = comas.max(puntos_y_comas).max(tabuladores).max(plecas);
    if max == 0 {
        return ',';
    }

    if plecas == max {
        '|'
    } else if tabuladores == max {
        '\t'
    } else if puntos_y_comas == max {
        ';'
    } else {
        ','
    }
}

/// Tokeniza texto CSV en filas y celdas respetando comillas y saltos de línea.
pub fn tokenizar_csv(texto: &str, delimitador: char) -> Vec<Vec<String>> {
    let mut filas = Vec::new();
    let mut fila_actual = Vec::new();
    let mut campo_actual = String::new();
    let mut en_comillas = false;
    let mut caracteres = texto.chars().peekable();

    while let Some(c) = caracteres.next() {
        if c == '"' {
            if en_comillas && caracteres.peek() == Some(&'"') {
                caracteres.next();
                campo_actual.push('"');
            } else {
                en_comillas = !en_comillas;
            }
        } else if c == delimitador && !en_comillas {
            fila_actual.push(campo_actual.trim().to_string());
            campo_actual = String::new();
        } else if (c == '\n' || c == '\r') && !en_comillas {
            if c == '\r' && caracteres.peek() == Some(&'\n') {
                caracteres.next();
            }
            fila_actual.push(campo_actual.trim().to_string());
            campo_actual = String::new();
            if !fila_actual.iter().all(|s| s.is_empty()) {
                filas.push(fila_actual);
            }
            fila_actual = Vec::new();
        } else {
            campo_actual.push(c);
        }
    }

    if !campo_actual.is_empty() || !fila_actual.is_empty() {
        fila_actual.push(campo_actual.trim().to_string());
        if !fila_actual.iter().all(|s| s.is_empty()) {
            filas.push(fila_actual);
        }
    }

    filas
}

/// Parsea un buffer de datos CSV y lo transforma en una lista de `Entrada`.
pub fn importar_csv(datos: &[u8], delimitador_manual: Option<char>) -> Result<Vec<Entrada>, BvdaError> {
    let mut texto = String::from_utf8_lossy(datos).to_string();
    if texto.starts_with('\u{feff}') {
        texto.remove(0); // Eliminar BOM UTF-8
    }

    let delimitador = delimitador_manual.unwrap_or_else(|| detectar_delimitador(&texto));
    let filas = tokenizar_csv(&texto, delimitador);

    if filas.len() < 2 {
        return Err(BvdaError::ErrorConfig("El archivo CSV no contiene suficientes filas de datos".into()));
    }

    let cabecera: Vec<String> = filas[0].iter().map(|s| s.to_lowercase()).collect();
    let idx_titulo = encontrar_indice(&cabecera, ALIAS_TITULO);
    let idx_url = encontrar_indice(&cabecera, ALIAS_URL);
    let idx_usuario = encontrar_indice(&cabecera, ALIAS_USUARIO);
    let idx_contrasena = encontrar_indice(&cabecera, ALIAS_CONTRASENA);
    let idx_notas = encontrar_indice(&cabecera, ALIAS_NOTAS);
    let idx_totp = encontrar_indice(&cabecera, ALIAS_TOTP);

    if idx_usuario.is_none() && idx_contrasena.is_none() {
        return Err(BvdaError::ErrorConfig("No se reconocen las columnas del archivo CSV".into()));
    }

    let ahora = Utc::now().timestamp_millis();
    let mut resultado = Vec::new();

    for (num, fila) in filas.into_iter().skip(1).enumerate() {
        let obtener = |idx_opt: Option<usize>| -> String {
            match idx_opt {
                Some(i) if i < fila.len() => fila[i].clone(),
                _ => String::new(),
            }
        };

        let usuario = obtener(idx_usuario);
        let contrasena = obtener(idx_contrasena);
        let url = obtener(idx_url);
        let mut titulo = obtener(idx_titulo);
        let notas = obtener(idx_notas);
        let totp_raw = obtener(idx_totp);

        if usuario.is_empty() && contrasena.is_empty() && url.is_empty() {
            continue;
        }

        if titulo.is_empty() {
            titulo = if !url.is_empty() {
                extraer_host_o_fallback(&url)
            } else if !usuario.is_empty() {
                usuario.clone()
            } else {
                format!("Entrada {}", num + 1)
            };
        }

        let urls = if !url.is_empty() { vec![url] } else { Vec::new() };
        let secreto_totp = if !totp_raw.is_empty() { Some(totp_raw) } else { None };

        resultado.push(Entrada {
            id: format!("csv-{}", num + 1),
            tipo: TipoEntrada::Login,
            titulo,
            usuario,
            contrasena,
            urls,
            notas,
            secreto_totp,
            creada_en: ahora,
            modificada_en: ahora,
            ..Default::default()
        });
    }

    Ok(resultado)
}

fn encontrar_indice(cabecera: &[String], aliases: &[&str]) -> Option<usize> {
    for alias in aliases {
        if let Some(pos) = cabecera.iter().position(|col| col == *alias || col.contains(alias)) {
            return Some(pos);
        }
    }
    None
}

fn extraer_host_o_fallback(url: &str) -> String {
    let limpio = url.trim().trim_start_matches("https://").trim_start_matches("http://");
    let host = limpio.split('/').next().unwrap_or(limpio);
    let host = host.split(':').next().unwrap_or(host);
    if !host.is_empty() {
        host.to_string()
    } else {
        url.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detectar_delimitador() {
        assert_eq!(detectar_delimitador("name,url,username,password"), ',');
        assert_eq!(detectar_delimitador("name;url;username;password"), ';');
        assert_eq!(detectar_delimitador("name\turl\tusername\tpassword"), '\t');
        assert_eq!(detectar_delimitador("name|url|username|password"), '|');
    }

    #[test]
    fn test_importar_csv_con_comas() {
        let csv = "name,url,username,password,notes\nGoogle,https://google.com,user@gmail.com,123456,Mi cuenta";
        let entradas = importar_csv(csv.as_bytes(), None).expect("importar csv");
        assert_eq!(entradas.len(), 1);
        assert_eq!(entradas[0].titulo, "Google");
        assert_eq!(entradas[0].usuario, "user@gmail.com");
        assert_eq!(entradas[0].contrasena, "123456");
        assert_eq!(entradas[0].notas, "Mi cuenta");
    }

    #[test]
    fn test_importar_csv_con_punto_y_coma_y_comillas() {
        let csv = "name;url;username;password;notes\n\"Mi Banco\";\"https://banco.es\";\"cliente1\";\"clave;secreta\";\"Notas con ; punto y coma\"";
        let entradas = importar_csv(csv.as_bytes(), None).expect("importar csv");
        assert_eq!(entradas.len(), 1);
        assert_eq!(entradas[0].titulo, "Mi Banco");
        assert_eq!(entradas[0].contrasena, "clave;secreta");
        assert_eq!(entradas[0].notas, "Notas con ; punto y coma");
    }
}
