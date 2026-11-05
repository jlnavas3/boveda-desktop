/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Aplicador de normalización de títulos en lote o para un grupo de servicio específico.

use crate::gui::service_grouper::extraer_nombre_servicio;
use crate::gui::state::titles::title_case_format::FormatoTitulo;
use crate::gui::state::titles::title_compromised_check::es_entrada_protegida_manual;
use crate::models::ContenidoBoveda;

pub fn aplicar_normalizacion_titulos(
    boveda: &mut ContenidoBoveda,
    formato_str: &str,
    respetar_personalizados: bool,
    clave_servicio_filtro: Option<&str>,
    titulo_manual: Option<&str>,
) -> usize {
    let formato = FormatoTitulo::desde_str(formato_str);
    let tlds = boveda
        .configuracion
        .as_ref()
        .map(|c| c.tlds_descartables.clone())
        .unwrap_or_default();
    let ahora = chrono::Utc::now().timestamp_millis();
    let mut modificadas = 0usize;

    for e in boveda.entradas.iter_mut().filter(|e| e.eliminada_en == 0) {
        let primer_url = e.urls.first().map(|s| s.as_str()).unwrap_or("");
        let serv_base = extraer_nombre_servicio(primer_url, &e.titulo);
        if serv_base.is_empty() {
            continue;
        }

        if respetar_personalizados && es_entrada_protegida_manual(e, &serv_base, &tlds) {
            continue;
        }

        if let Some(filtro_clave) = clave_servicio_filtro {
            if !serv_base.eq_ignore_ascii_case(filtro_clave) {
                continue;
            }
        }

        let destino = match titulo_manual {
            Some(m) if !m.trim().is_empty() => m.trim().to_string(),
            _ => formato.aplicar(&serv_base, primer_url),
        };

        if destino != e.titulo {
            e.titulo = destino;
            e.modificada_en = ahora;
            modificadas += 1;
        }
    }

    modificadas
}
