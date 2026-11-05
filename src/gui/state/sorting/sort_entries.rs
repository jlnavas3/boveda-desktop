/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Función pura desacoplada para ordenar referencias de entradas según el criterio activo.

use crate::gui::state::sorting::sort_criterion::CriterioOrdenacion;
use crate::models::Entrada;

pub fn ordenar_entradas_referencias(
    entradas: &mut [&Entrada],
    criterio_str: &str,
    fijar_anclados_arriba: bool,
) {
    let criterio = CriterioOrdenacion::desde_str(criterio_str);

    entradas.sort_by(|a, b| {
        if fijar_anclados_arriba && a.anclado != b.anclado {
            return b.anclado.cmp(&a.anclado);
        }

        let orden_primario = match criterio {
            CriterioOrdenacion::NombreAz => a.titulo.to_lowercase().cmp(&b.titulo.to_lowercase()),
            CriterioOrdenacion::NombreZa => b.titulo.to_lowercase().cmp(&a.titulo.to_lowercase()),
            CriterioOrdenacion::FechaModificacion => b.modificada_en.cmp(&a.modificada_en),
            CriterioOrdenacion::FechaCreacion => b.creada_en.cmp(&a.creada_en),
            CriterioOrdenacion::UltimoUso => b.ultimo_uso_en.cmp(&a.ultimo_uso_en),
            CriterioOrdenacion::FrecuenciaUso => {
                let peso_a = (a.favorito as u8) * 2 + (a.ultimo_uso_en > 0) as u8;
                let peso_b = (b.favorito as u8) * 2 + (b.ultimo_uso_en > 0) as u8;
                peso_b
                    .cmp(&peso_a)
                    .then_with(|| b.ultimo_uso_en.cmp(&a.ultimo_uso_en))
            }
        };

        orden_primario.then_with(|| a.titulo.to_lowercase().cmp(&b.titulo.to_lowercase()))
    });
}
