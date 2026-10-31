/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

use std::collections::{HashMap, HashSet};
use slint::SharedString;
use crate::gui::EntryOverview;

/// Extrae el nombre de servicio normalizado a partir de una URL o del título de la cuenta.
/// Remueve protocolos (https://, http://), subdominios comunes (www, login, etc.),
/// puertos, extensiones TLD conocidas (.com, .es, .co.uk, etc.) y normaliza a formato Capitalizado.
pub fn extraer_nombre_servicio(url: &str, titulo: &str) -> String {
    let url_limpia = url.trim();
    if !url_limpia.is_empty() {
        if let Some(servicio) = extraer_servicio_de_url(url_limpia) {
            if !servicio.is_empty() {
                return servicio;
            }
        }
    }

    extraer_servicio_de_titulo(titulo)
}

fn extraer_servicio_de_url(url: &str) -> Option<String> {
    let sin_esquema = if let Some(idx) = url.find("://") {
        &url[idx + 3..]
    } else {
        url
    };

    let host_completo = sin_esquema
        .split('/')
        .next()?
        .split('?')
        .next()?
        .split('#')
        .next()?
        .split(':')
        .next()?
        .trim();

    if host_completo.is_empty() {
        return None;
    }

    let partes: Vec<&str> = host_completo.split('.').collect();
    if partes.is_empty() {
        return None;
    }

    let subdominios_ignorados = [
        "www", "m", "mobile", "app", "apps", "login", "auth", "accounts", "account",
        "myaccount", "signin", "id", "portal", "secure", "api", "admin", "web",
    ];

    let tlds_compuestos = [
        "com.es", "co.uk", "com.ar", "com.mx", "com.br", "org.uk", "gov.uk", "edu.es", "org.es",
    ];

    let host_lower = host_completo.to_lowercase();
    let es_tld_compuesto = tlds_compuestos.iter().any(|tld| host_lower.ends_with(tld));

    let partes_filtradas: Vec<&str> = partes
        .into_iter()
        .filter(|p| !subdominios_ignorados.contains(&p.to_lowercase().as_str()))
        .collect();

    if partes_filtradas.is_empty() {
        return None;
    }

    let candidata = if partes_filtradas.len() == 1 {
        partes_filtradas[0]
    } else if es_tld_compuesto && partes_filtradas.len() >= 3 {
        partes_filtradas[partes_filtradas.len() - 3]
    } else {
        partes_filtradas[partes_filtradas.len() - 2]
    };

    Some(capitalizar_palabra(candidata))
}

fn extraer_servicio_de_titulo(titulo: &str) -> String {
    let t = titulo.trim();
    if t.is_empty() {
        return "General".to_string();
    }

    let primer_token = t
        .split(['(', '-', ':', '|', '/'])
        .next()
        .unwrap_or(t)
        .trim();

    if primer_token.is_empty() {
        return capitalizar_palabra(t);
    }

    let palabras: Vec<&str> = primer_token.split_whitespace().collect();
    if palabras.is_empty() {
        capitalizar_palabra(t)
    } else if palabras.len() == 1 {
        capitalizar_palabra(palabras[0])
    } else {
        format!("{} {}", capitalizar_palabra(palabras[0]), capitalizar_palabra(palabras[1]))
    }
}

fn capitalizar_palabra(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Agrupa las entradas según las reglas canónicas de Bóveda:
/// - Si `agrupar_activo == false`: no se crean grupos; se respetan anclados si está activo.
/// - Si `agrupar_activo == true`: si hay >= 2 entradas con el mismo nombre de servicio,
///   se crea una cabecera de grupo sintética y sus cuentas hijas se posicionan de forma integrada contigua.
///   El último hijo se marca con `es_ultimo_hijo_grupo = true` para cerrar visualmente el paquete único.
/// - Entradas con cuenta única se muestran normalmente como ítems sueltos.
/// - Si `fijar_anclados_arriba == true`: las cuentas ancladas se procesan al inicio sin agrupar.
pub fn aplicar_agrupamiento_servicios(
    entradas: &[EntryOverview],
    agrupar_activo: bool,
    fijar_anclados_arriba: bool,
    grupos_expandidos: &HashSet<String>,
) -> Vec<EntryOverview> {
    if entradas.is_empty() {
        return Vec::new();
    }

    // Si fijar_anclados_arriba está activo, separamos las ancladas
    let (mut ancladas, no_ancladas): (Vec<EntryOverview>, Vec<EntryOverview>) = if fijar_anclados_arriba {
        entradas.iter().cloned().partition(|e| e.anclado)
    } else {
        (Vec::new(), entradas.to_vec())
    };

    if !agrupar_activo {
        let mut resultado = Vec::with_capacity(entradas.len());
        for mut e in ancladas {
            e.es_cabecera_grupo = false;
            e.es_hijo_grupo = false;
            e.es_ultimo_hijo_grupo = false;
            resultado.push(e);
        }
        for mut e in no_ancladas {
            e.es_cabecera_grupo = false;
            e.es_hijo_grupo = false;
            e.es_ultimo_hijo_grupo = false;
            resultado.push(e);
        }
        return resultado;
    }

    // Contar ocurrencias por servicio en no_ancladas
    let mut conteos: HashMap<String, usize> = HashMap::new();
    for e in &no_ancladas {
        let servicio = if e.servicio.is_empty() {
            extraer_nombre_servicio("", &e.titulo)
        } else {
            e.servicio.to_string()
        };
        *conteos.entry(servicio).or_insert(0) += 1;
    }

    let mut resultado = Vec::with_capacity(entradas.len() + 10);

    // Agregar las entradas ancladas (individuales en el tope)
    for mut e in ancladas.drain(..) {
        e.es_cabecera_grupo = false;
        e.es_hijo_grupo = false;
        e.es_ultimo_hijo_grupo = false;
        resultado.push(e);
    }

    // Procesar no_ancladas agrupando juntos los ítems del mismo servicio
    let mut grupos_procesados: HashSet<String> = HashSet::new();

    for e in &no_ancladas {
        let servicio = if e.servicio.is_empty() {
            extraer_nombre_servicio("", &e.titulo)
        } else {
            e.servicio.to_string()
        };

        if grupos_procesados.contains(&servicio) {
            continue;
        }

        let conteo = *conteos.get(&servicio).unwrap_or(&1);

        if conteo >= 2 {
            grupos_procesados.insert(servicio.clone());
            let expandido = grupos_expandidos.contains(&servicio);

            let cabecera = EntryOverview {
                id: SharedString::from(format!("grp:{}", servicio)),
                titulo: SharedString::from(&servicio),
                subtitulo: SharedString::from(format!("{} cuentas", conteo)),
                tipo: SharedString::from("grupo"),
                favorito: false,
                anclado: false,
                categoria_id: SharedString::from(""),
                tiene_usuario: false,
                tiene_password: false,
                tiene_totp: false,
                tiene_passkey: false,
                tiene_url: false,
                tiene_app: false,
                tiene_notas: false,
                totp_codigo: SharedString::from(""),
                totp_restante: 0,
                servicio: SharedString::from(&servicio),
                es_cabecera_grupo: true,
                clave_grupo: SharedString::from(&servicio),
                grupo_conteo: conteo as i32,
                grupo_expandido: expandido,
                es_hijo_grupo: false,
                es_ultimo_hijo_grupo: false,
            };
            resultado.push(cabecera);

            // Si el grupo está expandido, agregamos todos sus hijos contiguos
            if expandido {
                let hijos: Vec<EntryOverview> = no_ancladas
                    .iter()
                    .filter(|item| {
                        let item_serv = if item.servicio.is_empty() {
                            extraer_nombre_servicio("", &item.titulo)
                        } else {
                            item.servicio.to_string()
                        };
                        item_serv == servicio
                    })
                    .cloned()
                    .collect();

                let total_hijos = hijos.len();
                for (idx, mut hijo) in hijos.into_iter().enumerate() {
                    hijo.es_cabecera_grupo = false;
                    hijo.es_hijo_grupo = true;
                    hijo.es_ultimo_hijo_grupo = idx == total_hijos - 1;
                    hijo.clave_grupo = SharedString::from(&servicio);
                    resultado.push(hijo);
                }
            }
        } else {
            // Entrada individual (suelta)
            let mut solitaria = e.clone();
            solitaria.es_cabecera_grupo = false;
            solitaria.es_hijo_grupo = false;
            solitaria.es_ultimo_hijo_grupo = false;
            solitaria.clave_grupo = SharedString::from("");
            resultado.push(solitaria);
        }
    }

    resultado
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extraer_nombre_servicio_url() {
        assert_eq!(extraer_nombre_servicio("https://www.amazon.es/login", "Amazon"), "Amazon");
        assert_eq!(extraer_nombre_servicio("https://login.amazon.com", "Mi Cuenta"), "Amazon");
        assert_eq!(extraer_nombre_servicio("https://github.com/settings", "Git"), "Github");
        assert_eq!(extraer_nombre_servicio("https://netflix.com", "Netflix"), "Netflix");
        assert_eq!(extraer_nombre_servicio("", "Amazon (Personal)"), "Amazon");
        assert_eq!(extraer_nombre_servicio("", "Google - Trabajo"), "Google");
    }

    #[test]
    fn test_agrupamiento_servicios() {
        let entradas = vec![
            EntryOverview {
                id: SharedString::from("1"),
                titulo: SharedString::from("Amazon Personal"),
                subtitulo: SharedString::from("user1"),
                tipo: SharedString::from("login"),
                favorito: false,
                anclado: false,
                categoria_id: SharedString::from(""),
                tiene_usuario: true,
                tiene_password: true,
                tiene_totp: false,
                tiene_passkey: false,
                tiene_url: false,
                tiene_app: false,
                tiene_notas: false,
                totp_codigo: SharedString::from(""),
                totp_restante: 0,
                servicio: SharedString::from("Amazon"),
                es_cabecera_grupo: false,
                clave_grupo: SharedString::from(""),
                grupo_conteo: 0,
                grupo_expandido: false,
                es_hijo_grupo: false,
                es_ultimo_hijo_grupo: false,
            },
            EntryOverview {
                id: SharedString::from("2"),
                titulo: SharedString::from("Amazon Trabajo"),
                subtitulo: SharedString::from("user2"),
                tipo: SharedString::from("login"),
                favorito: false,
                anclado: false,
                categoria_id: SharedString::from(""),
                tiene_usuario: true,
                tiene_password: true,
                tiene_totp: false,
                tiene_passkey: false,
                tiene_url: false,
                tiene_app: false,
                tiene_notas: false,
                totp_codigo: SharedString::from(""),
                totp_restante: 0,
                servicio: SharedString::from("Amazon"),
                es_cabecera_grupo: false,
                clave_grupo: SharedString::from(""),
                grupo_conteo: 0,
                grupo_expandido: false,
                es_hijo_grupo: false,
                es_ultimo_hijo_grupo: false,
            },
            EntryOverview {
                id: SharedString::from("3"),
                titulo: SharedString::from("GitHub"),
                subtitulo: SharedString::from("user3"),
                tipo: SharedString::from("login"),
                favorito: false,
                anclado: false,
                categoria_id: SharedString::from(""),
                tiene_usuario: true,
                tiene_password: true,
                tiene_totp: false,
                tiene_passkey: false,
                tiene_url: false,
                tiene_app: false,
                tiene_notas: false,
                totp_codigo: SharedString::from(""),
                totp_restante: 0,
                servicio: SharedString::from("Github"),
                es_cabecera_grupo: false,
                clave_grupo: SharedString::from(""),
                grupo_conteo: 0,
                grupo_expandido: false,
                es_hijo_grupo: false,
                es_ultimo_hijo_grupo: false,
            },
        ];

        let mut expandidos = HashSet::new();

        // 1. Colapsado
        let res_colapsado = aplicar_agrupamiento_servicios(&entradas, true, false, &expandidos);
        assert_eq!(res_colapsado.len(), 2);
        assert!(res_colapsado[0].es_cabecera_grupo);
        assert_eq!(res_colapsado[0].titulo.as_str(), "Amazon");
        assert_eq!(res_colapsado[0].grupo_conteo, 2);
        assert!(!res_colapsado[1].es_cabecera_grupo);
        assert_eq!(res_colapsado[1].titulo.as_str(), "GitHub");

        // 2. Expandido
        expandidos.insert("Amazon".to_string());
        let res_expandido = aplicar_agrupamiento_servicios(&entradas, true, false, &expandidos);
        assert_eq!(res_expandido.len(), 4);
        assert!(res_expandido[0].es_cabecera_grupo);
        assert!(res_expandido[1].es_hijo_grupo);
        assert!(!res_expandido[1].es_ultimo_hijo_grupo);
        assert!(res_expandido[2].es_hijo_grupo);
        assert!(res_expandido[2].es_ultimo_hijo_grupo);
        assert!(!res_expandido[3].es_hijo_grupo);
    }
}
