/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Construye la lista reactiva de Perfiles / Identidades para el gestor CRUD y la barra lateral.

use slint::SharedString;

use crate::gui::profiles::resolver_perfil_para_entrada;
use crate::gui::ProfileManagerItem;
use crate::models::ContenidoBoveda;

pub fn listar_perfiles_items(boveda: &ContenidoBoveda) -> Vec<ProfileManagerItem> {
    boveda
        .perfiles
        .iter()
        .map(|p| {
            let conteo = boveda
                .entradas
                .iter()
                .filter(|e| {
                    e.eliminada_en == 0
                        && resolver_perfil_para_entrada(e, &boveda.perfiles)
                            .map(|res| res.id == p.id)
                            .unwrap_or(false)
                })
                .count() as i32;

            ProfileManagerItem {
                id: SharedString::from(&p.id),
                nombre: SharedString::from(&p.nombre),
                correo: SharedString::from(&p.correo_principal),
                correos_secundarios: SharedString::from(p.correos_secundarios.join(", ")),
                tipo_documento: SharedString::from(p.tipo_documento.as_deref().unwrap_or("CI")),
                documento: SharedString::from(p.numero_documento.as_deref().unwrap_or("")),
                preferencia_identificador: SharedString::from(&p.preferencia_identificador),
                conteo,
                conteo_docs: p.documentos_ids.len() as i32,
                conteo_tarjetas: p.tarjetas_ids.len() as i32,
                conteo_cuentas: p.cuentas_ids.len() as i32,
                conteo_contactos: p.contactos_ids.len() as i32,
            }
        })
        .collect()
}
