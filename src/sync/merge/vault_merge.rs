/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Fusión completa de dos contenedores de bóveda (ContenidoBoveda).
//! Resuelve colisiones por marcas temporales y aplica la fusión aditiva sin pérdida.

use crate::models::{Categoria, ContenidoBoveda, Entrada, Perfil};
use crate::sync::merge::entry_merge::fusionar_entradas;
use std::collections::HashMap;

/// Fusiona dos bóvedas completas sin pérdida de datos.
pub fn fusionar_bovedas(boveda_a: &ContenidoBoveda, boveda_b: &ContenidoBoveda) -> ContenidoBoveda {
    // 1. Fusión de categorías (colecciones)
    let mut mapa_categorias: HashMap<String, Categoria> = HashMap::new();
    for cat in &boveda_a.categorias {
        mapa_categorias.insert(cat.id.clone(), cat.clone());
    }
    for cat in &boveda_b.categorias {
        match mapa_categorias.get_mut(&cat.id) {
            Some(existente) => {
                if cat.modificada_en > existente.modificada_en {
                    *existente = cat.clone();
                }
            }
            None => {
                mapa_categorias.insert(cat.id.clone(), cat.clone());
            }
        }
    }

    // 2. Fusión de perfiles (identidades)
    let mut mapa_perfiles: HashMap<String, Perfil> = HashMap::new();
    for perf in &boveda_a.perfiles {
        mapa_perfiles.insert(perf.id.clone(), perf.clone());
    }
    for perf in &boveda_b.perfiles {
        match mapa_perfiles.get_mut(&perf.id) {
            Some(existente) => {
                if perf.modificada_en > existente.modificada_en {
                    *existente = perf.clone();
                }
            }
            None => {
                mapa_perfiles.insert(perf.id.clone(), perf.clone());
            }
        }
    }

    // 3. Fusión de entradas activas
    let mut mapa_entradas: HashMap<String, Entrada> = HashMap::new();
    for e in &boveda_a.entradas {
        mapa_entradas.insert(e.id.clone(), e.clone());
    }

    for e_b in &boveda_b.entradas {
        match mapa_entradas.remove(&e_b.id) {
            Some(e_a) => {
                // Colisión de ID: la más reciente es la principal
                let (principal, secundaria) = if e_b.modificada_en >= e_a.modificada_en {
                    (e_b, &e_a)
                } else {
                    (&e_a, e_b)
                };
                let fusionada = fusionar_entradas(principal, std::slice::from_ref(secundaria));
                mapa_entradas.insert(fusionada.id.clone(), fusionada);
            }
            None => {
                mapa_entradas.insert(e_b.id.clone(), e_b.clone());
            }
        }
    }

    // 4. Fusión de papelera
    let mut mapa_papelera: HashMap<String, Entrada> = HashMap::new();
    for p in &boveda_a.papelera {
        mapa_papelera.insert(p.id.clone(), p.clone());
    }
    for p_b in &boveda_b.papelera {
        match mapa_papelera.get_mut(&p_b.id) {
            Some(existente) => {
                if p_b.eliminada_en > existente.eliminada_en || p_b.modificada_en > existente.modificada_en {
                    *existente = p_b.clone();
                }
            }
            None => {
                mapa_papelera.insert(p_b.id.clone(), p_b.clone());
            }
        }
    }

    // Resolver colisión entre activa y papelera: si una activa tiene modificada_en posterior a eliminada_en, permanece activa
    for (id, p) in &mapa_papelera {
        if let Some(activa) = mapa_entradas.get(id) {
            if activa.modificada_en >= p.eliminada_en {
                // La activa es más reciente, se descarta de la papelera
            } else {
                // La eliminación fue posterior, se remueve de activas
                mapa_entradas.remove(id);
            }
        }
    }

    let mut entradas_finales: Vec<Entrada> = mapa_entradas.into_values().collect();
    entradas_finales.sort_by(|a, b| a.titulo.cmp(&b.titulo));

    let mut categorias_finales: Vec<Categoria> = mapa_categorias.into_values().collect();
    categorias_finales.sort_by(|a, b| a.nombre.cmp(&b.nombre));

    let mut perfiles_finales: Vec<Perfil> = mapa_perfiles.into_values().collect();
    perfiles_finales.sort_by(|a, b| a.nombre.cmp(&b.nombre));

    let papelera_final: Vec<Entrada> = mapa_papelera.into_values().collect();

    // Configuración: preferir la de la bóveda B si existe, o A
    let configuracion_final = boveda_b.configuracion.clone().or_else(|| boveda_a.configuracion.clone());

    ContenidoBoveda {
        version: boveda_a.version.max(boveda_b.version),
        entradas: entradas_finales,
        papelera: papelera_final,
        categorias: categorias_finales,
        perfiles: perfiles_finales,
        configuracion: configuracion_final,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fusion_bovedas_disjuntas() {
        let mut a = ContenidoBoveda::default();
        a.entradas.push(Entrada {
            id: "1".into(),
            titulo: "Cuenta A".into(),
            ..Default::default()
        });

        let mut b = ContenidoBoveda::default();
        b.entradas.push(Entrada {
            id: "2".into(),
            titulo: "Cuenta B".into(),
            ..Default::default()
        });

        let res = fusionar_bovedas(&a, &b);
        assert_eq!(res.entradas.len(), 2);
    }

    #[test]
    fn test_fusion_bovedas_con_colision_de_id() {
        let mut a = ContenidoBoveda::default();
        a.entradas.push(Entrada {
            id: "1".into(),
            titulo: "Google".into(),
            usuario: "user@gmail.com".into(),
            contrasena: "claveVieja".into(),
            modificada_en: 1000,
            ..Default::default()
        });

        let mut b = ContenidoBoveda::default();
        b.entradas.push(Entrada {
            id: "1".into(),
            titulo: "Google".into(),
            usuario: "user@gmail.com".into(),
            contrasena: "claveNueva2026".into(),
            modificada_en: 2000, // Más reciente
            ..Default::default()
        });

        let res = fusionar_bovedas(&a, &b);
        assert_eq!(res.entradas.len(), 1);
        let e = &res.entradas[0];
        assert_eq!(e.contrasena, "claveNueva2026");
        assert_eq!(e.historial_contrasenas.len(), 1);
        assert_eq!(e.historial_contrasenas[0].contrasena, "claveVieja");
    }
}
