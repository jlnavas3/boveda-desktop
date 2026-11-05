/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Fusión completa de dos contenedores de bóveda (ContenidoBoveda).
//! Orquestador determinista de sincronización y fusión bidireccional de bóvedas completas.
//! Espejo canónico de `FusionadorBovedas.kt` en Android.

use crate::models::{Categoria, ContenidoBoveda, Entrada, Perfil};
use crate::sync::merge::arbiter::{evaluar_estado_entrada, DecisionArbitro};
use crate::sync::merge::diff_mask::{calcular_diff, de_entrada_nueva};
use crate::sync::merge::entry_hasher::calcular_content_hash;
use crate::sync::merge::entry_merge::fusionar_entradas;
use crate::sync::merge::sync_report::{InformeSincronizacion, ItemCambioSync, TipoCambioSync};
use std::collections::HashMap;

/// Fusiona dos bóvedas completas emitiendo en el mismo pase el `InformeSincronizacion`
/// detallado con las máscaras de campos modificados relativo a `boveda_local`.
pub fn fusionar_bovedas_con_informe(
    boveda_local: &ContenidoBoveda,
    boveda_remota: &ContenidoBoveda,
) -> (ContenidoBoveda, InformeSincronizacion) {
    // 1. Fusión de categorías (colecciones) por modificada_en
    let mut mapa_categorias: HashMap<String, Categoria> = HashMap::new();
    for cat in &boveda_local.categorias {
        mapa_categorias.insert(cat.id.clone(), cat.clone());
    }
    for cat in &boveda_remota.categorias {
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

    // 2. Fusión de perfiles (identidades) por modificada_en
    let mut mapa_perfiles: HashMap<String, Perfil> = HashMap::new();
    for perf in &boveda_local.perfiles {
        mapa_perfiles.insert(perf.id.clone(), perf.clone());
    }
    for perf in &boveda_remota.perfiles {
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

    // 3. Fusión de entradas (universo completo: activas + papelera)
    let mut todas_locales: HashMap<String, Entrada> = HashMap::new();
    for e in boveda_local.entradas.iter().chain(boveda_local.papelera.iter()) {
        todas_locales.insert(e.id.clone(), e.clone());
    }

    let mut todas_remotas: HashMap<String, Entrada> = HashMap::new();
    for e in boveda_remota.entradas.iter().chain(boveda_remota.papelera.iter()) {
        todas_remotas.insert(e.id.clone(), e.clone());
    }

    let mut entradas_fusionadas: HashMap<String, Entrada> = HashMap::new();
    let mut lista_nuevas = Vec::new();
    let mut lista_actualizadas = Vec::new();
    let mut lista_papelera = Vec::new();
    let mut sin_cambios_count = 0;

    for (id, remota) in todas_remotas {
        if let Some(local) = todas_locales.remove(&id) {
            let hash_local = calcular_content_hash(&local);
            let hash_remoto = calcular_content_hash(&remota);

            let decision = evaluar_estado_entrada(
                &hash_local,
                &local.last_sync_hash,
                local.revision,
                &hash_remoto,
                &remota.last_sync_hash,
                remota.revision,
            );

            let entrada_resuelta = match decision {
                DecisionArbitro::SinCambios | DecisionArbitro::FastForwardLocal => {
                    let mut e = local.clone();
                    e.last_sync_hash = Some(hash_local);
                    e
                }
                DecisionArbitro::FastForwardRemoto => {
                    let mut e = remota;
                    e.last_sync_hash = Some(hash_remoto);
                    e
                }
                DecisionArbitro::SplitBrain => {
                    let (principal, secundaria) = if remota.modificada_en >= local.modificada_en {
                        (&remota, &local)
                    } else {
                        (&local, &remota)
                    };
                    let mut fusionada = fusionar_entradas(principal, std::slice::from_ref(secundaria));
                    let nueva_rev = local.revision.max(remota.revision) + 1;
                    fusionada.revision = nueva_rev;
                    let nuevo_hash = calcular_content_hash(&fusionada);
                    fusionada.last_sync_hash = Some(nuevo_hash);
                    fusionada
                }
            };
            entradas_fusionadas.insert(id, entrada_resuelta.clone());

            // Evaluación en el mismo pase del informe relativo a boveda_local
            let mascara = calcular_diff(&local, &entrada_resuelta);
            if local.eliminada_en > 0 && entrada_resuelta.eliminada_en == 0 {
                lista_papelera.push(ItemCambioSync {
                    id: entrada_resuelta.id,
                    titulo: entrada_resuelta.titulo,
                    tipo_cambio: TipoCambioSync::RestauradaDePapelera,
                    mascara,
                });
            } else if local.eliminada_en == 0 && entrada_resuelta.eliminada_en > 0 {
                lista_papelera.push(ItemCambioSync {
                    id: entrada_resuelta.id,
                    titulo: entrada_resuelta.titulo,
                    tipo_cambio: TipoCambioSync::EnviadaAPapelera,
                    mascara,
                });
            } else if mascara.hay_cambios() || local.titulo != entrada_resuelta.titulo {
                lista_actualizadas.push(ItemCambioSync {
                    id: entrada_resuelta.id,
                    titulo: entrada_resuelta.titulo,
                    tipo_cambio: TipoCambioSync::Actualizada,
                    mascara,
                });
            } else {
                sin_cambios_count += 1;
            }
        } else {
            // Entrada solo existente en remota (100% nueva para local)
            let hash = calcular_content_hash(&remota);
            let mut e = remota;
            e.last_sync_hash = Some(hash);
            entradas_fusionadas.insert(id, e.clone());

            let mascara = de_entrada_nueva(&e);
            lista_nuevas.push(ItemCambioSync {
                id: e.id,
                titulo: e.titulo,
                tipo_cambio: TipoCambioSync::Nueva,
                mascara,
            });
        }
    }

    // Entradas restantes que solo existen en local (100% conservadas sin cambios para local)
    for (id, local) in todas_locales {
        let hash = calcular_content_hash(&local);
        let mut e = local;
        e.last_sync_hash = Some(hash);
        entradas_fusionadas.insert(id, e);
        sin_cambios_count += 1;
    }

    // 4. Partición estricta entre activas y papelera
    let mut entradas_finales: Vec<Entrada> = entradas_fusionadas
        .values()
        .filter(|e| e.eliminada_en == 0)
        .cloned()
        .collect();
    entradas_finales.sort_by_key(|a| a.titulo.to_lowercase());

    let mut papelera_final: Vec<Entrada> = entradas_fusionadas
        .values()
        .filter(|e| e.eliminada_en > 0)
        .cloned()
        .collect();
    papelera_final.sort_by_key(|a| a.titulo.to_lowercase());

    let mut categorias_finales: Vec<Categoria> = mapa_categorias.into_values().collect();
    categorias_finales.sort_by_key(|a| a.nombre.to_lowercase());

    let mut perfiles_finales: Vec<Perfil> = mapa_perfiles.into_values().collect();
    perfiles_finales.sort_by_key(|a| a.nombre.to_lowercase());

    // Configuración: preferir la de la bóveda remota si existe, o local
    let configuracion_final = boveda_remota.configuracion.clone().or_else(|| boveda_local.configuracion.clone());

    let boveda_final = ContenidoBoveda {
        version: boveda_local.version.max(boveda_remota.version),
        entradas: entradas_finales,
        papelera: papelera_final,
        categorias: categorias_finales,
        perfiles: perfiles_finales,
        configuracion: configuracion_final,
        alias_dispositivo: None,
    };

    let informe = InformeSincronizacion {
        nuevas: lista_nuevas,
        actualizadas: lista_actualizadas,
        papelera: lista_papelera,
        sin_cambios_count,
        dispositivo_remoto: boveda_remota.alias_dispositivo.clone(),
    };

    (boveda_final, informe)
}

/// Fusiona dos bóvedas completas (`boveda_a` y `boveda_b`) de manera determinista y sin pérdida de datos.
pub fn fusionar_bovedas(boveda_a: &ContenidoBoveda, boveda_b: &ContenidoBoveda) -> ContenidoBoveda {
    fusionar_bovedas_con_informe(boveda_a, boveda_b).0
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
