/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Medidor de fuerza de contraseñas soberano.
//! Réplica matemática exacta de `MedidorFuerza.kt` de Android utilizando zxcvbn y entropía combinatoria.

use zxcvbn::zxcvbn;

/// Resultado de la evaluación de fuerza de una contraseña.
#[derive(Debug, Clone, PartialEq)]
pub struct FuerzaContrasena {
    pub puntuacion: u8,
    pub etiqueta: &'static str,
    pub bits: f64,
}

const MAYUSCULAS_LEN: usize = 24; // "ABCDEFGHJKLMNPQRSTUVWXYZ"
const MINUSCULAS_LEN: usize = 25; // "abcdefghijkmnopqrstuvwxyz"
const DIGITOS_LEN: usize = 8;     // "23456789"
const SIMBOLOS_LEN: usize = 23;   // "!@#$%&*()-_=+[]{}?/.,:;"

/// Evalúa la fuerza de una contraseña replicando idénticamente `MedidorFuerza.medir` de Android.
pub fn medir_fuerza_contrasena(contrasena: &str) -> FuerzaContrasena {
    if contrasena.is_empty() {
        return FuerzaContrasena {
            puntuacion: 0,
            etiqueta: "Vacía",
            bits: 0.0,
        };
    }

    let recortada: String = contrasena.chars().take(72).collect();
    let medida = zxcvbn(&recortada, &[]);
    let score = u8::from(medida.score());
    let intentos = (medida.guesses() as f64).max(1.0);
    let bits_zxcvbn = intentos.ln() / 2.0_f64.ln();

    let bits = if score >= 3 {
        let tiene_ambiguos = contrasena.chars().any(|c| c == 'I' || c == 'O' || c == 'l' || c == '0' || c == '1');
        let mut pool = 0;
        if contrasena.chars().any(|c| c.is_ascii_uppercase()) {
            pool += if tiene_ambiguos { 26 } else { MAYUSCULAS_LEN };
        }
        if contrasena.chars().any(|c| c.is_ascii_lowercase()) {
            pool += if tiene_ambiguos { 26 } else { MINUSCULAS_LEN };
        }
        if contrasena.chars().any(|c| c.is_ascii_digit()) {
            pool += if tiene_ambiguos { 10 } else { DIGITOS_LEN };
        }
        if contrasena.chars().any(|c| !c.is_alphanumeric()) {
            pool += if tiene_ambiguos { 33 } else { SIMBOLOS_LEN };
        }

        let bits_combinatorios = if pool > 0 {
            contrasena.chars().count() as f64 * ((pool as f64).ln() / 2.0_f64.ln())
        } else {
            0.0
        };

        bits_zxcvbn.max(bits_combinatorios)
    } else {
        bits_zxcvbn
    };

    let etiqueta = if bits >= 90.0 || score == 4 {
        "Excelente"
    } else if bits >= 65.0 || score == 3 {
        "Fuerte"
    } else if bits >= 45.0 || score == 2 {
        "Aceptable"
    } else if score == 1 {
        "Débil"
    } else {
        "Muy débil"
    };

    let puntuacion = match etiqueta {
        "Excelente" => 4,
        "Fuerte" => 3,
        "Aceptable" => 2,
        "Débil" => 1,
        _ => 0,
    };

    FuerzaContrasena {
        puntuacion,
        etiqueta,
        bits,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_medir_fuerza() {
        let debil = medir_fuerza_contrasena("123456");
        assert!(debil.puntuacion <= 1);

        let fuerte = medir_fuerza_contrasena("C0mpl3j@-Y-Soberana-2026!");
        assert!(fuerte.puntuacion >= 3);
    }
}
