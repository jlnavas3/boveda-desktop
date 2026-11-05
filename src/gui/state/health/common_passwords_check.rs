/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Verificador soberano de contraseñas comunes o filtradas en diccionarios públicos.
//! Paridad 100% con `contrasenas_comunes.txt` de Android.

pub fn es_contrasena_comun(contrasena: &str) -> bool {
    crate::crypto::es_contrasena_comun(contrasena)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comunes() {
        assert!(es_contrasena_comun("password"));
        assert!(es_contrasena_comun("123456"));
        assert!(!es_contrasena_comun("ClaveCompletamenteUnica2026!#$"));
    }
}
