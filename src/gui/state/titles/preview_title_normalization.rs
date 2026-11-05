/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Generador de previsualización de grupos de títulos que pueden ser normalizados.

use std::collections::BTreeMap;
use slint::SharedString;

use crate::gui::service_grouper::extraer_nombre_servicio;
use crate::gui::state::titles::title_case_format::FormatoTitulo;
use crate::gui::state::titles::title_compromised_check::{
    es_entrada_protegida_manual, es_titulo_comprometido,
};
use crate::gui::TitleNormalizationGroupItem;
use crate::models::ContenidoBoveda;

pub fn previsualizar_normalizacion_titulos(
    boveda: &ContenidoBoveda,
    formato_str: &str,
    respetar_personalizados: bool,
) -> Vec<TitleNormalizationGroupItem> {
    let formato = FormatoTitulo::desde_str(formato_str);
    let tlds = boveda
        .configuracion
        .as_ref()
        .map(|c| c.tlds_descartables.clone())
        .unwrap_or_default();

    let mut grupos: BTreeMap<String, (String, Vec<String>, usize)> = BTreeMap::new();

    for e in boveda.entradas.iter().filter(|e| e.eliminada_en == 0) {
        let primer_url = e.urls.first().map(|s| s.as_str()).unwrap_or("");
        let serv_base = extraer_nombre_servicio(primer_url, &e.titulo);
        if serv_base.is_empty() {
            continue;
        }

        // Si respetar_personalizados está activo, comprobamos protección manual canónica
        if respetar_personalizados && es_entrada_protegida_manual(e, &serv_base, &tlds) {
            continue;
        }

        // En Android soloComprometidos es true por defecto para evitar renombrar títulos limpios
        if !es_titulo_comprometido(e, &tlds) && respetar_personalizados {
            continue;
        }

        let propuesto = formato.aplicar(&serv_base, primer_url);
        if propuesto != e.titulo {
            let clave = serv_base.to_lowercase();
            let entry = grupos
                .entry(clave)
                .or_insert_with(|| (propuesto.clone(), Vec::new(), 0));
            entry.2 += 1;
            if entry.1.len() < 3 && !entry.1.contains(&e.titulo) {
                entry.1.push(e.titulo.clone());
            }
        }
    }

    grupos
        .into_iter()
        .map(|(clave, (propuesto, ejemplos, conteo))| TitleNormalizationGroupItem {
            clave_servicio: SharedString::from(clave),
            titulo_propuesto: SharedString::from(propuesto),
            ejemplos_antes: SharedString::from(ejemplos.join(", ")),
            conteo_afectadas: conteo as i32,
        })
        .collect()
}
