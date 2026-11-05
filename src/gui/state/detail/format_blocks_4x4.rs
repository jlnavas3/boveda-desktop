/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Formateador puro de cadenas alfanuméricas en bloques de 4 caracteres (`4x4`) para tarjetas PAN e IBAN.

pub fn formatear_bloques_4x4(valor: &str, activo: bool) -> String {
    if !activo {
        return valor.to_string();
    }
    let limpio: String = valor.chars().filter(|c| !c.is_whitespace()).collect();
    if limpio.len() <= 4 {
        return limpio;
    }

    let mut resultado = String::with_capacity(limpio.len() + limpio.len() / 4);
    for (idx, ch) in limpio.chars().enumerate() {
        if idx > 0 && idx % 4 == 0 {
            resultado.push(' ');
        }
        resultado.push(ch);
    }
    resultado
}
