/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

/// Convierte valores HSV (Tono 0.0..360.0, Saturación 0.0..1.0, Brillo 0.0..1.0)
/// al espacio sRGB (0..255) y devuelve su representación hexadecimal `#RRGGBB`.
pub fn hsv_a_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8, String) {
    let h_norm = (h % 360.0 + 360.0) % 360.0;
    let s_norm = s.clamp(0.0, 1.0);
    let v_norm = v.clamp(0.0, 1.0);

    let c = v_norm * s_norm;
    let x = c * (1.0 - ((h_norm / 60.0) % 2.0 - 1.0).abs());
    let m = v_norm - c;

    let (r_prime, g_prime, b_prime) = if h_norm < 60.0 {
        (c, x, 0.0)
    } else if h_norm < 120.0 {
        (x, c, 0.0)
    } else if h_norm < 180.0 {
        (0.0, c, x)
    } else if h_norm < 240.0 {
        (0.0, x, c)
    } else if h_norm < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    let r = ((r_prime + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    let g = ((g_prime + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    let b = ((b_prime + m) * 255.0).round().clamp(0.0, 255.0) as u8;

    let hex = format!("#{r:02X}{g:02X}{b:02X}");
    (r, g, b, hex)
}

/// Convierte valores sRGB (0..255) al espacio HSV (Tono 0.0..360.0, Saturación 0.0..1.0, Brillo 0.0..1.0).
pub fn rgb_a_hsv(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let r_norm = r as f32 / 255.0;
    let g_norm = g as f32 / 255.0;
    let b_norm = b as f32 / 255.0;

    let max = r_norm.max(g_norm).max(b_norm);
    let min = r_norm.min(g_norm).min(b_norm);
    let delta = max - min;

    let h = if delta == 0.0 {
        0.0
    } else if (max - r_norm).abs() < f32::EPSILON {
        ((g_norm - b_norm) / delta * 60.0 + 360.0) % 360.0
    } else if (max - g_norm).abs() < f32::EPSILON {
        (b_norm - r_norm) / delta * 60.0 + 120.0
    } else {
        (r_norm - g_norm) / delta * 60.0 + 240.0
    };

    let s = if max == 0.0 { 0.0 } else { delta / max };
    let v = max;

    (h, s, v)
}

/// Convierte una cadena Hexadecimal (`#RRGGBB` o `RRGGBB`) a HSV.
pub fn hex_a_hsv(hex: &str) -> Option<(f32, f32, f32)> {
    let limpio = hex.trim().trim_start_matches('#');
    if limpio.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&limpio[0..2], 16).ok()?;
    let g = u8::from_str_radix(&limpio[2..4], 16).ok()?;
    let b = u8::from_str_radix(&limpio[4..6], 16).ok()?;
    Some(rgb_a_hsv(r, g, b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rojo_puro() {
        let (r, g, b, hex) = hsv_a_rgb(0.0, 1.0, 1.0);
        assert_eq!((r, g, b), (255, 0, 0));
        assert_eq!(hex, "#FF0000");

        let (h, s, v) = rgb_a_hsv(255, 0, 0);
        assert_eq!(h, 0.0);
        assert!((s - 1.0).abs() < 1e-4);
        assert!((v - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_azul_usuario_canonic() {
        // #2563EB -> r: 37, g: 99, b: 235
        let (h, s, v) = rgb_a_hsv(37, 99, 235);
        let (r, g, b, hex) = hsv_a_rgb(h, s, v);
        assert_eq!(hex, "#2563EB");
        assert_eq!((r, g, b), (37, 99, 235));
    }

    #[test]
    fn test_amarillo_password_canonic() {
        // #EAB308 -> r: 234, g: 179, b: 8
        let (h, s, v) = rgb_a_hsv(234, 179, 8);
        let (r, g, b, hex) = hsv_a_rgb(h, s, v);
        assert_eq!(hex, "#EAB308");
        assert_eq!((r, g, b), (234, 179, 8));
    }

    #[test]
    fn test_hex_a_hsv_valido() {
        let res = hex_a_hsv("#06B6D4");
        assert!(res.is_some());
        let (h, s, v) = res.unwrap();
        let (_, _, _, hex) = hsv_a_rgb(h, s, v);
        assert_eq!(hex, "#06B6D4");
    }

    #[test]
    fn test_hex_a_hsv_invalido() {
        assert!(hex_a_hsv("xyz").is_none());
        assert!(hex_a_hsv("#12345").is_none());
    }
}
