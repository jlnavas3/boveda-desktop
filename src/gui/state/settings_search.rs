/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Buscador desacoplado de subsecciones en Ajustes de Bóveda.
//! Devuelve `(categoria_activa, subseccion_activa)` cuando el término coincide.

pub fn buscar_subseccion_ajustes(termino: &str) -> Option<(i32, i32)> {
    let q = termino.trim().to_lowercase();
    if q.is_empty() {
        return None;
    }

    const MAPA_BUSQUEDA: &[(&[&str], i32, i32)] = &[
        (&["temporizador", "autobloqueo", "portapapeles", "freno", "inactividad"], 1, 1),
        (&["argon", "argon2", "kdf", "memoria", "iteracion"], 1, 2),
        (&["clave maestra", "contraseña maestra", "master"], 1, 3),
        (&["autodestruccion", "biometria", "huella", "intentos"], 1, 4),
        (&["comunes", "diccionario", "filtradas"], 1, 5),
        (&["tema", "oscuro", "claro", "acento", "paleta"], 2, 101),
        (&["tipografia", "fuente", "texto", "letra"], 2, 102),
        (&["forma", "borde", "esquina", "curvatura", "geometria"], 2, 103),
        (&["menu lateral", "sidebar", "cabecera", "pie"], 2, 104),
        (&["diseño lista", "densidad"], 2, 105),
        (&["niagara", "abecedario", "ola", "indice"], 2, 106),
        (&["formato", "4x4", "iban", "fecha", "tarjeta"], 2, 107),
        (&["plantilla", "preset"], 2, 108),
        (&["indicador", "icono", "color"], 2, 109),
        (&["nombre", "identidad boveda"], 2, 110),
        (&["aviso", "campana", "asistencia"], 2, 111),
        (&["animacion", "transicion"], 2, 112),
        (&["credencial", "resumen"], 3, 201),
        (&["orden", "fijar", "anclado", "reciente"], 3, 202),
        (&["privacidad", "ocultar", "desenfoque"], 3, 203),
        (&["salud", "auditoria", "zxcvbn", "debil", "repetida"], 3, 204),
        (&["papelera", "reciclaje", "vaciar", "retencion"], 3, 205),
        (&["categoria", "carpeta", "perfil"], 3, 206),
        (&["generador", "aleatoria", "frase"], 4, 301),
        (&["totp", "2fa", "autenticador", "verificacion"], 4, 302),
        (&["historial"], 4, 303),
        (&["puerto", "homelab", "proxmox", "servicio"], 4, 304),
        (&["subdominio", "normalizar", "titulo", "url"], 4, 305),
        (&["exportar", "copia", "json", "csv", "respaldo"], 5, 401),
        (&["importar", "bitwarden", "chrome"], 5, 402),
        (&["fusion", "merge", "unir"], 5, 403),
        (&["kit", "emergencia", "kit de emergencia", "papel", "rescate"], 5, 404),
        (&["escritorio", "zoom", "resolucion", "ventana"], 6, 501),
        (&["diagnostico", "cripto", "aes"], 6, 502),
        (&["registro", "evento", "log", "bitacora"], 6, 503),
        (&["acerca", "version", "licencia", "autor"], 6, 504),
    ];

    // 1. Coincidencia directa donde una palabra clave contiene la consulta
    for (keywords, cat, sub) in MAPA_BUSQUEDA {
        if keywords.iter().any(|k| k.contains(&q)) {
            return Some((*cat, *sub));
        }
    }

    // 2. Coincidencia por tokens individuales de la consulta
    let palabras: Vec<&str> = q.split_whitespace().collect();
    for (keywords, cat, sub) in MAPA_BUSQUEDA {
        if keywords
            .iter()
            .any(|k| palabras.iter().any(|w| *k == *w || k.starts_with(w)))
        {
            return Some((*cat, *sub));
        }
    }

    None
}
