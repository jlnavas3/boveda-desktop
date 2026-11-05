/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Conteo canónico de entradas secundarias duplicadas para el badge de navegación.
//! Réplica fiel en Rust de `AnalizadorDuplicados.analizar(entradas).sumOf { it.entradasSecundarias.size }`.

use crate::gui::state::duplicates::detect_duplicate_groups::detectar_grupos_duplicados;
use crate::models::ContenidoBoveda;

/// Cuenta el total de entradas secundarias a unificar/limpiar en duplicados.
/// En Android: `AnalizadorDuplicados.analizar(entradas).sumOf { it.entradasSecundarias.size }`.
/// Si no hay grupos o cada grupo solo tiene su principal, el conteo es 0.
pub fn contar_entradas_secundarias_duplicadas(boveda: &ContenidoBoveda) -> i32 {
    let grupos = detectar_grupos_duplicados(boveda, "TODOS");
    let mut total_secundarias = 0;
    for g in grupos {
        if g.conteo_entradas > 1 {
            total_secundarias += g.conteo_entradas - 1;
        }
    }
    total_secundarias
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contar_entradas_secundarias_vacio() {
        let boveda = ContenidoBoveda::default();
        assert_eq!(contar_entradas_secundarias_duplicadas(&boveda), 0);
    }
}
