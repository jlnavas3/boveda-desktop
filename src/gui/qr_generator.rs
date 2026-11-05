/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Generador canónico de imágenes QR en memoria para Slint UI.
//! Convierte datos de texto o URI en un búfer de píxeles RGBA nítido y de alto contraste.

use qrcode::{Color, QrCode};
use slint::{Image, Rgba8Pixel, SharedPixelBuffer};

/// Genera una imagen QR de alta resolución lista para renderizarse en Slint UI.
///
/// Aplica una zona de silencio (quiet zone) de 4 módulos y escala cada módulo a 8x8 píxeles
/// para garantizar lectura instantánea por cámaras móviles.
pub fn generar_qr_imagen(datos: &str) -> Result<Image, String> {
    let code = QrCode::new(datos.as_bytes())
        .map_err(|e| format!("Error generando matriz QR: {e}"))?;

    let qr_width = code.width();
    let quiet_zone = 4;
    let total_modules = qr_width + quiet_zone * 2;
    let module_size = 8;
    let img_size = (total_modules * module_size) as u32;

    let mut pixel_buffer = SharedPixelBuffer::<Rgba8Pixel>::new(img_size, img_size);
    let slice = pixel_buffer.make_mut_slice();

    // Fondo blanco nítido (#FFFFFF)
    for pixel in slice.iter_mut() {
        *pixel = Rgba8Pixel {
            r: 255,
            g: 255,
            b: 255,
            a: 255,
        };
    }

    let colors = code.to_colors();
    for (i, color) in colors.into_iter().enumerate() {
        if color == Color::Dark {
            let mod_x = (i % qr_width) + quiet_zone;
            let mod_y = (i / qr_width) + quiet_zone;

            let start_px = mod_x * module_size;
            let start_py = mod_y * module_size;

            for dy in 0..module_size {
                for dx in 0..module_size {
                    let px = (start_px + dx) as u32;
                    let py = (start_py + dy) as u32;
                    let idx = (py * img_size + px) as usize;
                    // Color oscuro nítido (#0F172A slate-900)
                    slice[idx] = Rgba8Pixel {
                        r: 15,
                        g: 23,
                        b: 42,
                        a: 255,
                    };
                }
            }
        }
    }

    Ok(Image::from_rgba8(pixel_buffer))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generar_qr_imagen_valida() {
        let uri = "bvda-sync://192.168.1.50:52424?k=010203&t=040506&v=1";
        let res = generar_qr_imagen(uri);
        assert!(res.is_ok());
        let img = res.unwrap();
        let size = img.size();
        assert!(size.width > 0);
        assert!(size.height > 0);
    }
}
