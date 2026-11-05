/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Calculador canónico del conteo consolidado de problemas de salud de contraseñas.
//! Réplica fiel en Rust de `contarProblemasSaludBoveda` (`CalculadorSaludBoveda.kt` de Android).

use std::collections::{HashMap, HashSet};

use crate::crypto::{es_contrasena_comun, medir_fuerza_contrasena};
use crate::models::{ContenidoBoveda, Entrada, TipoEntrada};

/// Calcula de forma consistente el número consolidado de cuentas con problemas
/// de salud (repetidas + débiles sin duplicación), idéntico a Android:
/// `(repetidas + debiles).size`
pub fn contar_problemas_salud_boveda_canonico(boveda: &ContenidoBoveda) -> i32 {
    let activas: Vec<&Entrada> = boveda
        .entradas
        .iter()
        .filter(|e| e.eliminada_en == 0 && e.tipo == TipoEntrada::Login && !e.ignorada_en_salud)
        .collect();

    // Contabilizar repeticiones
    let mut mapa_rep: HashMap<&str, usize> = HashMap::new();
    for e in &activas {
        let pw = e.contrasena.trim();
        if !pw.is_empty() {
            *mapa_rep.entry(pw).or_insert(0) += 1;
        }
    }

    let mut problemas: HashSet<&str> = HashSet::new();

    for e in &activas {
        let pw = e.contrasena.trim();
        if pw.is_empty() {
            continue;
        }

        // 1. Repetidas
        if let Some(&rep) = mapa_rep.get(pw) {
            if rep > 1 {
                problemas.insert(e.id.as_str());
            }
        }

        // 2. Débiles (longitud < 8 o puntuación fuerza <= 1 idéntico a Android con zxcvbn)
        let f = medir_fuerza_contrasena(pw);
        let es_debil = pw.chars().count() < 8 || f.puntuacion <= 1 || es_contrasena_comun(pw);

        if es_debil {
            problemas.insert(e.id.as_str());
        }
    }

    problemas.len() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conteo_salud_canonico_union_sin_duplicidad() {
        let mut boveda = ContenidoBoveda::default();
        boveda.entradas.push(Entrada {
            id: "1".into(),
            tipo: TipoEntrada::Login,
            contrasena: "123456".into(), // Débil, común y repetida
            ..Default::default()
        });
        boveda.entradas.push(Entrada {
            id: "2".into(),
            tipo: TipoEntrada::Login,
            contrasena: "123456".into(), // Repetida
            ..Default::default()
        });
        boveda.entradas.push(Entrada {
            id: "3".into(),
            tipo: TipoEntrada::Login,
            contrasena: "ClaveUltraFuerte2026!#@X".into(), // Sana
            ..Default::default()
        });

        // 2 cuentas afectadas en total, no se suman 3 ni 4 veces
        assert_eq!(contar_problemas_salud_boveda_canonico(&boveda), 2);
    }
}
