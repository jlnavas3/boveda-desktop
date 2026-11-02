/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

slint::include_modules!();

pub mod clipboard;
pub mod color_hsv;
pub mod service_grouper;
pub mod state;
pub mod zoom;

pub use clipboard::ClipboardManager;
pub use color_hsv::{hex_a_hsv, hsv_a_rgb, rgb_a_hsv};
pub use service_grouper::{aplicar_agrupamiento_servicios, extraer_nombre_servicio};
pub use state::{
    agregar_puerto_a_str, agregar_subdominio_a_str, detectar_indicadores, edit_data_a_entrada,
    eliminar_puerto_de_str, eliminar_subdominio_de_str, entrada_a_edit_data, parse_puertos_filas,
    parse_puertos_filas_con_ancho, parse_puertos_items, parse_subdominios_filas,
    parse_subdominios_filas_con_ancho, parse_subdominios_items, resolver_tema,
    subtitulo_segun_tipo, ColoresTemaResueltos, GuiVaultState, IndicadoresEntrada, ModoTema,
    PaletaAcentoInfo, PALETAS_ACENTO_OFICIALES,
};
pub use zoom::{aplicar_zoom_ventana, calcular_escala_automatica, ZoomState};
