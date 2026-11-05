/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

slint::include_modules!();

pub mod clipboard;
pub mod color_hsv;
pub mod domain;
pub mod profiles;
pub mod qr_generator;
pub mod service_grouper;
pub mod state;
pub mod zoom;

pub use clipboard::ClipboardManager;
pub use color_hsv::{hex_a_hsv, hsv_a_rgb, rgb_a_hsv};
pub use qr_generator::generar_qr_imagen;
pub use service_grouper::{aplicar_agrupamiento_servicios, extraer_nombre_servicio};
pub use state::{
    agregar_puerto_a_str, agregar_subdominio_a_str, alternar_anclado_lote,
    alternar_favorito_lote, alternar_ignorar_salud_lote, analizar_discrepancias_unificacion,
    analizar_salud_boveda, aplicar_normalizacion_titulos, asignar_categorias_lote,
    buscar_subseccion_ajustes, construir_detalle_entrada, contar_entradas_secundarias_duplicadas,
    contar_problemas_salud_boveda_canonico, detectar_grupos_duplicados, detectar_indicadores,
    edit_data_a_entrada, eliminar_categoria_boveda, eliminar_contrasena_historial,
    eliminar_passkey_secundaria, eliminar_perfil_boveda, eliminar_puerto_de_str,
    eliminar_subdominio_de_str, eliminar_totp_secundario, entrada_a_edit_data, es_contrasena_comun,
    formatear_bloques_4x4, generar_kit_emergencia_texto, guardar_o_crear_categoria,
    guardar_o_crear_perfil, listar_perfiles_items, ordenar_entradas_referencias,
    parse_puertos_filas, parse_puertos_filas_con_ancho, parse_puertos_items,
    parse_subdominios_filas, parse_subdominios_filas_con_ancho, parse_subdominios_items,
    previsualizar_normalizacion_titulos, procesar_papelera_lote, promover_passkey_secundaria,
    promover_totp_secundario, renombrar_lote, resolver_tema, restaurar_contrasena_historial,
    rotar_contrasena_rapida, subtitulo_segun_tipo, unificar_entradas_seleccionadas,
    unificar_grupo_duplicado, unificar_todos_los_duplicados, ColoresTemaResueltos,
    CriterioOrdenacion, EstadoSeleccionLista, FiltroDuplicados, FormatoTitulo, GuiVaultState,
    IndicadoresEntrada, ModoTema, PaletaAcentoInfo, PestanaSalud, PALETAS_ACENTO_OFICIALES,
};
pub use zoom::{aplicar_zoom_ventana, calcular_escala_automatica, ZoomState};
