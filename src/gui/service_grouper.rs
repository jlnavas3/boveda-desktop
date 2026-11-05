/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

use std::collections::{HashMap, HashSet};
use slint::SharedString;
use crate::gui::domain::service_extractor::extraer_nombre_servicio_canonico;
use crate::gui::EntryOverview;

/// Extrae el nombre de servicio normalizado a partir de una URL o del título de la cuenta
/// utilizando el motor canónico compatible con Android (`NormalizadorTitulosSitios.kt`).
pub fn extraer_nombre_servicio(url: &str, titulo: &str) -> String {
    extraer_nombre_servicio_canonico(url, titulo)
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
    let (ancladas, no_ancladas): (Vec<EntryOverview>, Vec<EntryOverview>) = if fijar_anclados_arriba {
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

    // Agregar las entradas ancladas (agrupadas en bloque plegable idéntico a Android)
    if !ancladas.is_empty() {
        let expandido = grupos_expandidos.contains("__ANCLADOS__");
        let n_anc = ancladas.len();
        let sub_texto = if n_anc == 1 {
            "1 anclada".to_string()
        } else {
            format!("{n_anc} ancladas")
        };
        let anclados_seleccionados = !ancladas.is_empty() && ancladas.iter().all(|m| m.seleccionada_lote);

        let cabecera_anclados = EntryOverview {
            id: SharedString::from("grp:__ANCLADOS__"),
            titulo: SharedString::from("Anclados"),
            subtitulo: SharedString::from(sub_texto),
            tipo: SharedString::from("grupo"),
            favorito: false,
            anclado: true,
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
            servicio: SharedString::from("Anclados"),
            es_cabecera_grupo: true,
            clave_grupo: SharedString::from("__ANCLADOS__"),
            grupo_conteo: n_anc as i32,
            grupo_expandido: expandido,
            es_hijo_grupo: false,
            es_ultimo_hijo_grupo: false,
            seleccionada_lote: anclados_seleccionados,
            mostrar_chip_perfil: false,
            nombre_perfil: SharedString::default(),
            color_perfil: slint::Color::from_argb_u8(255, 59, 130, 246),
            es_documento_perfil: false,
        };
        resultado.push(cabecera_anclados);

        if expandido {
            let total_hijos = ancladas.len();
            for (idx, mut hijo) in ancladas.into_iter().enumerate() {
                hijo.es_cabecera_grupo = false;
                hijo.es_hijo_grupo = true;
                hijo.es_ultimo_hijo_grupo = idx == total_hijos - 1;
                hijo.clave_grupo = SharedString::from("__ANCLADOS__");
                resultado.push(hijo);
            }
        }
    }

    // Procesar no_ancladas agrupando juntos los ítems del mismo servicio
    let mut grupos_procesados: HashSet<String> = HashSet::new();
    let mut bloques_alfabeticos: Vec<(String, Vec<EntryOverview>)> = Vec::new();

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
            let mut bloque = Vec::with_capacity(if expandido { conteo + 1 } else { 1 });

            let miembros_servicio: Vec<EntryOverview> = no_ancladas
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
            let grupo_seleccionado =
                !miembros_servicio.is_empty() && miembros_servicio.iter().all(|m| m.seleccionada_lote);

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
                seleccionada_lote: grupo_seleccionado,
                mostrar_chip_perfil: false,
                nombre_perfil: SharedString::default(),
                color_perfil: slint::Color::from_argb_u8(255, 59, 130, 246),
                es_documento_perfil: false,
            };
            bloque.push(cabecera);

            // Si el grupo está expandido, agregamos todos sus hijos contiguos
            if expandido {
                let total_hijos = miembros_servicio.len();
                for (idx, mut hijo) in miembros_servicio.into_iter().enumerate() {
                    hijo.es_cabecera_grupo = false;
                    hijo.es_hijo_grupo = true;
                    hijo.es_ultimo_hijo_grupo = idx == total_hijos - 1;
                    hijo.clave_grupo = SharedString::from(&servicio);
                    bloque.push(hijo);
                }
            }

            bloques_alfabeticos.push((servicio.to_lowercase(), bloque));
        } else {
            // Entrada individual (suelta)
            let mut solitaria = e.clone();
            solitaria.es_cabecera_grupo = false;
            solitaria.es_hijo_grupo = false;
            solitaria.es_ultimo_hijo_grupo = false;
            solitaria.clave_grupo = SharedString::from("");
            bloques_alfabeticos.push((solitaria.titulo.to_lowercase(), vec![solitaria]));
        }
    }

    // Ordenar bloques (grupos y entradas sueltas) estrictamente por su título visible en la lista
    bloques_alfabeticos.sort_by(|a, b| a.0.cmp(&b.0));

    for (_, bloque) in bloques_alfabeticos {
        resultado.extend(bloque);
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
        assert_eq!(extraer_nombre_servicio("https://github.com/settings", "Git"), "GitHub");
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
                servicio: SharedString::from("Amazon"),
                ..Default::default()
            },
            EntryOverview {
                id: SharedString::from("2"),
                titulo: SharedString::from("Amazon Trabajo"),
                subtitulo: SharedString::from("user2"),
                servicio: SharedString::from("Amazon"),
                ..Default::default()
            },
            EntryOverview {
                id: SharedString::from("3"),
                titulo: SharedString::from("GitHub"),
                subtitulo: SharedString::from("user3"),
                servicio: SharedString::from("Github"),
                ..Default::default()
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

    #[test]
    fn test_orden_alfabetico_de_grupos_por_nombre_servicio() {
        let e1 = EntryOverview {
            id: SharedString::from("1"),
            titulo: SharedString::from("Agencia Tributaria"),
            servicio: SharedString::from("Gob"),
            ..Default::default()
        };

        let e2 = EntryOverview {
            id: SharedString::from("2"),
            titulo: SharedString::from("Aeropost"),
            servicio: SharedString::from("Aeropost"),
            ..Default::default()
        };

        let e3 = EntryOverview {
            id: SharedString::from("3"),
            titulo: SharedString::from("Ministerio"),
            servicio: SharedString::from("Gob"),
            ..Default::default()
        };

        let entradas = vec![e1, e2, e3];
        let expandidos = HashSet::new();
        let res = aplicar_agrupamiento_servicios(&entradas, true, false, &expandidos);

        assert_eq!(res.len(), 2);
        assert_eq!(res[0].titulo.as_str(), "Aeropost");
        assert!(!res[0].es_cabecera_grupo);
        assert_eq!(res[1].titulo.as_str(), "Gob");
        assert!(res[1].es_cabecera_grupo);
    }
}
