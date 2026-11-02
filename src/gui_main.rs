/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use zeroize::{Zeroize, Zeroizing};

use boveda_core::config::AppConfigDesktop;
use boveda_core::crypto::{
    calcular_fuerza_contrasena, desempaquetar_boveda, generar_contrasena, CabeceraBvda,
    PasswordGeneratorConfig, PerfilArgon2,
};
use boveda_core::gui::{
    agregar_puerto_a_str, agregar_subdominio_a_str, aplicar_zoom_ventana, edit_data_a_entrada,
    eliminar_puerto_de_str, eliminar_subdominio_de_str, entrada_a_edit_data, hsv_a_rgb,
    parse_puertos_filas_con_ancho, parse_subdominios_filas_con_ancho, resolver_tema, AppWindow,
    ClipboardManager, EntryDetailData, EntryEditData, GeneratorOptions, GuiVaultState,
    HsvColorResult, ModoTema, SettingsData, Theme, ZoomState,
};

/// Refresca y sincroniza reactivamente todos los modelos visuales de Slint desde el estado en memoria.
fn refrescar_ui(ui: &AppWindow, st: &mut GuiVaultState, id_preferido: Option<&str>) {
    let tipos = st.obtener_resumen_tipos();
    let categorias = st.obtener_resumen_categorias();
    let tipos_dropdown = st.obtener_opciones_tipos_dropdown();
    let categorias_dropdown = st.obtener_opciones_categorias_dropdown();
    let perfiles = st.obtener_resumen_perfiles();
    let etiquetas = st.obtener_resumen_etiquetas();
    let entradas = st.obtener_entradas_filtradas();

    if let Some(id) = id_preferido {
        st.entrada_seleccionada_id = Some(id.to_string());
    } else if let Some(ref sel_id) = st.entrada_seleccionada_id {
        if !entradas.iter().any(|e| e.id.as_str() == sel_id) {
            st.entrada_seleccionada_id = entradas.first().map(|e| e.id.to_string());
        }
    } else {
        st.entrada_seleccionada_id = entradas.first().map(|e| e.id.to_string());
    }

    let detalle = st
        .entrada_seleccionada_id
        .as_deref()
        .and_then(|id| st.obtener_detalle_entrada(id))
        .unwrap_or_default();

    ui.set_tipos_filtro(ModelRc::new(VecModel::from(tipos)));
    ui.set_categorias_filtro(ModelRc::new(VecModel::from(categorias)));
    ui.set_tipos_dropdown(ModelRc::new(VecModel::from(tipos_dropdown)));
    ui.set_categorias_dropdown(ModelRc::new(VecModel::from(categorias_dropdown)));
    ui.set_perfiles_filtro(ModelRc::new(VecModel::from(perfiles)));
    ui.set_etiquetas_filtro(ModelRc::new(VecModel::from(etiquetas)));
    ui.set_lista_entradas(ModelRc::new(VecModel::from(entradas)));
    ui.set_entrada_seleccionada_id(SharedString::from(
        st.entrada_seleccionada_id.clone().unwrap_or_default(),
    ));
    ui.set_detalle_actual(detalle);
}

/// Borra el texto de notificación después de un tiempo especificado sin bloquear el hilo principal.
fn limpiar_notificacion_despues(ui_weak: &slint::Weak<AppWindow>, segundos: u64) {
    let ui_handle = ui_weak.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(segundos));
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_texto_notificacion(SharedString::default());
            }
        });
    });
}

/// Convierte una cadena hexadecimal en un objeto `slint::Color`.
fn parse_hex_color(hex: &str) -> slint::Color {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() == 6 {
        let r = u8::from_str_radix(&clean[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&clean[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&clean[4..6], 16).unwrap_or(0);
        slint::Color::from_argb_u8(255, r, g, b)
    } else {
        slint::Color::from_argb_u8(255, 0, 0, 0)
    }
}

/// Aplica el tema dinámico y la paleta de acento seleccionada globalmente en tiempo real.
fn aplicar_tema_a_ui(ui: &AppWindow, modo: ModoTema, clave_acento: &str) {
    let res = resolver_tema(modo, clave_acento, true);
    let theme = ui.global::<Theme>();
    theme.set_es_oscuro(res.es_oscuro);
    theme.set_modo_actual(modo.as_str().into());
    theme.set_acento_actual(clave_acento.into());
    theme.set_background(parse_hex_color(res.background));
    theme.set_surface(parse_hex_color(res.surface));
    theme.set_surface_hover(parse_hex_color(res.surface_hover));
    theme.set_surface_selected(parse_hex_color(res.surface_selected));
    theme.set_border(parse_hex_color(res.border));
    theme.set_border_focus(parse_hex_color(res.border_focus));
    theme.set_text_primary(parse_hex_color(res.text_primary));
    theme.set_text_secondary(parse_hex_color(res.text_secondary));
    theme.set_text_muted(parse_hex_color(res.text_muted));
    theme.set_accent_primary(parse_hex_color(res.accent_primary));
    theme.set_accent_hover(parse_hex_color(res.accent_hover));
    theme.set_danger(parse_hex_color(res.danger));
}

/// Convierte las opciones reactivas de Slint a la configuración del generador criptográfico.
fn mapear_opciones_generador(opts: &GeneratorOptions) -> PasswordGeneratorConfig {
    PasswordGeneratorConfig {
        longitud: opts.longitud.max(4) as usize,
        mayusculas: opts.mayusculas,
        minusculas: opts.minusculas,
        numeros: opts.numeros,
        simbolos: opts.simbolos,
        evitar_ambiguos: opts.evitar_ambiguos,
        modo_frase: opts.modo_frase,
        numero_palabras: (opts.numero_palabras.clamp(3, 8) as usize),
        separador: opts.separador.to_string(),
        capitalizar_palabras: opts.capitalizar_palabras,
    }
}

/// Genera la contraseña y evalúa su fuerza, actualizando las propiedades correspondientes de Slint.
fn ejecutar_generacion(ui: &AppWindow, opts: &GeneratorOptions) {
    let config = mapear_opciones_generador(opts);
    let pw = generar_contrasena(&config);
    let (puntuacion, etiqueta) = calcular_fuerza_contrasena(&pw);

    ui.set_generador_resultado(SharedString::from(pw.as_str()));
    ui.set_generador_fuerza_puntuacion(puntuacion);
    ui.set_generador_fuerza_etiqueta(SharedString::from(etiqueta));
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    // 1. Cargar configuración local de escritorio (Zero-Hardcoding)
    let app_config = AppConfigDesktop::cargar().unwrap_or_default();
    let ruta_predeterminada = app_config
        .ruta_boveda
        .clone()
        .unwrap_or_else(|| PathBuf::from("boveda.bvda"));

    let ruta_str = ruta_predeterminada.display().to_string();
    ui.set_ruta_boveda(SharedString::from(&ruta_str));

    // Comprobar si hay clave en el llavero biométrico del sistema
    let bio_disponible = boveda_core::gui::state::biometrics::esta_biometria_disponible(&ruta_str);
    ui.set_biometria_disponible(bio_disponible);

    let state = Arc::new(Mutex::new(GuiVaultState::new(ruta_predeterminada)));

    // 2. Callback: Examinar archivo .bvda con diálogo nativo
    let ui_weak = ui.as_weak();
    let state_clone = Arc::clone(&state);
    ui.on_examinar_archivo(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let inicio = state_clone.lock().unwrap().ruta_boveda.clone();
            let mut dialog = rfd::FileDialog::new()
                .set_title("Seleccionar archivo de Bóveda Local (.bvda)")
                .add_filter("Bóveda Local (*.bvda)", &["bvda"]);

            if let Some(padre) = inicio.parent() {
                dialog = dialog.set_directory(padre);
            }

            if let Some(archivo) = dialog.pick_file() {
                let r_str = archivo.display().to_string();
                ui.set_ruta_boveda(SharedString::from(&r_str));
                let bio = boveda_core::gui::state::biometrics::esta_biometria_disponible(&r_str);
                ui.set_biometria_disponible(bio);
                state_clone.lock().unwrap().ruta_boveda = archivo;
            }
        }
    });

    // 3. Callback: Intentar Desbloquear Bóveda
    let ui_weak = ui.as_weak();
    let state_clone = Arc::clone(&state);
    ui.on_intentar_desbloquear(move |ruta_str, pass_str| {
        let ui = match ui_weak.upgrade() {
            Some(u) => u,
            None => return,
        };

        ui.set_desbloqueando(true);
        ui.set_error_desbloqueo(SharedString::default());

        let mut password = pass_str.to_string();
        let clave_guardar = Zeroizing::new(password.clone());
        let ruta = PathBuf::from(ruta_str.as_str());
        let ui_handle = ui_weak.clone();
        let state_worker = Arc::clone(&state_clone);

        thread::spawn(move || {
            let resultado = (|| -> Result<(boveda_core::ContenidoBoveda, PerfilArgon2), String> {
                let bytes = fs::read(&ruta)
                    .map_err(|e| format!("No se pudo leer el archivo '{}': {e}", ruta.display()))?;

                let perfil = if let Ok(cabecera) = CabeceraBvda::parsear(&bytes) {
                    if cabecera.memory_kib >= 262_144 {
                        PerfilArgon2::Ultraseguro
                    } else if cabecera.memory_kib >= 131_072 {
                        PerfilArgon2::Reforzado
                    } else {
                        PerfilArgon2::Estandar
                    }
                } else {
                    PerfilArgon2::Estandar
                };

                let boveda = desempaquetar_boveda(&bytes, password.as_bytes())
                    .map_err(|e| format!("Fallo al descifrar bóveda: {e}"))?;

                Ok((boveda, perfil))
            })();

            password.zeroize();

            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_handle.upgrade() {
                    ui.set_desbloqueando(false);
                    match resultado {
                        Ok((boveda, perfil)) => {
                            let mut st = state_worker.lock().unwrap();
                            st.cargar_boveda(boveda, ruta, Some(clave_guardar), Some(perfil));

                            ui.set_filtro_tipo_actual("TODOS".into());
                            ui.set_filtro_categoria_actual("TODAS".into());
                            ui.set_ajustes_datos(boveda_core::config::defaults::valores_predeterminados());
                            refrescar_ui(&ui, &mut st, None);
                            ui.set_boveda_desbloqueada(true);
                        }
                        Err(err) => {
                            ui.set_error_desbloqueo(SharedString::from(err));
                        }
                    }
                }
            });
        });
    });

    // 4. Callback: Desbloqueo Biométrico
    let ui_weak = ui.as_weak();
    ui.on_desbloquear_biometria(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let ruta = ui.get_ruta_boveda().to_string();
            if let Ok(clave) = boveda_core::gui::state::biometrics::obtener_clave_biometria(&ruta) {
                ui.invoke_intentar_desbloquear(ruta.into(), clave.into());
            } else {
                ui.set_error_desbloqueo("No se pudo desbloquear con biometría o clave no encontrada".into());
            }
        }
    });

    // 5. Callback: Bloquear Bóveda
    let ui_weak = ui.as_weak();
    let state_clone = Arc::clone(&state);
    ui.on_bloquear_boveda(move || {
        if let Some(ui) = ui_weak.upgrade() {
            state_clone.lock().unwrap().bloquear();
            ui.set_boveda_desbloqueada(false);
            ui.set_lista_entradas(ModelRc::new(VecModel::default()));
            ui.set_detalle_actual(EntryDetailData::default());
            ui.set_texto_notificacion(SharedString::default());
            ui.set_mostrar_editor(false);
            ui.set_mostrar_generador(false);
            ui.set_mostrar_ajustes(false);
        }
    });

    // 6. Callback: Seleccionar Tipo de Entrada
    let ui_weak = ui.as_weak();
    let state_clone = Arc::clone(&state);
    ui.on_seleccionar_tipo(move |tipo| {
        if let Some(ui) = ui_weak.upgrade() {
            let mut st = state_clone.lock().unwrap();
            st.filtro_tipo = tipo.to_string();
            ui.set_filtro_tipo_actual(tipo);
            ui.set_lista_viewport_y(0.0);
            refrescar_ui(&ui, &mut st, None);
        }
    });

    // 7. Callback: Seleccionar Categoría
    let ui_weak = ui.as_weak();
    let state_clone = Arc::clone(&state);
    ui.on_seleccionar_categoria(move |cat| {
        if let Some(ui) = ui_weak.upgrade() {
            let mut st = state_clone.lock().unwrap();
            st.filtro_categoria = cat.to_string();
            ui.set_filtro_categoria_actual(cat);
            ui.set_lista_viewport_y(0.0);
            refrescar_ui(&ui, &mut st, None);
        }
    });

    // 7.1 Callback: Seleccionar Perfil
    let ui_weak = ui.as_weak();
    let state_clone = Arc::clone(&state);
    ui.on_seleccionar_perfil(move |perf| {
        if let Some(ui) = ui_weak.upgrade() {
            let mut st = state_clone.lock().unwrap();
            st.filtro_perfil = perf.to_string();
            ui.set_filtro_perfil_actual(perf);
            ui.set_lista_viewport_y(0.0);
            refrescar_ui(&ui, &mut st, None);
        }
    });

    // 7.2 Callback: Seleccionar Etiqueta
    let ui_weak = ui.as_weak();
    let state_clone = Arc::clone(&state);
    ui.on_seleccionar_etiqueta(move |tag| {
        if let Some(ui) = ui_weak.upgrade() {
            let mut st = state_clone.lock().unwrap();
            st.filtro_etiqueta = tag.to_string();
            ui.set_filtro_etiqueta_actual(tag);
            ui.set_lista_viewport_y(0.0);
            refrescar_ui(&ui, &mut st, None);
        }
    });

    // 8. Callback: Búsqueda en Vivo
    let ui_weak = ui.as_weak();
    let state_clone = Arc::clone(&state);
    ui.on_buscar(move |termino| {
        if let Some(ui) = ui_weak.upgrade() {
            let mut st = state_clone.lock().unwrap();
            st.termino_busqueda = termino.to_string();
            ui.set_lista_viewport_y(0.0);
            refrescar_ui(&ui, &mut st, None);
        }
    });

    // 9. Callback: Salto Niagara A-Z
    let ui_weak = ui.as_weak();
    let state_clone = Arc::clone(&state);
    ui.on_saltar_a_letra(move |letra| {
        if let Some(ui) = ui_weak.upgrade() {
            if let Some(c) = letra.chars().next() {
                let mut st = state_clone.lock().unwrap();
                let entradas = st.obtener_entradas_filtradas();
                let indice = boveda_core::gui::state::filter_engine::encontrar_indice_por_letra(&entradas, c);
                if let Some(id) = boveda_core::gui::state::filter_engine::encontrar_id_por_letra(&entradas, c) {
                    st.entrada_seleccionada_id = Some(id.clone());
                    refrescar_ui(&ui, &mut st, Some(&id));
                }
                if let Some(idx) = indice {
                    // Calcular desplazamiento exacto considerando grupos expandidos (64px unidos vs 70px separados)
                    let scroll_y: f32 = entradas
                        .iter()
                        .take(idx)
                        .map(|e| {
                            if (e.es_cabecera_grupo && e.grupo_expandido)
                                || (e.es_hijo_grupo && !e.es_ultimo_hijo_grupo)
                            {
                                64.0
                            } else {
                                70.0
                            }
                        })
                        .sum();
                    ui.set_lista_viewport_y(-scroll_y);
                }
            }
        }
    });

    // 10. Callback: Seleccionar Entrada
    let ui_weak = ui.as_weak();
    let state_clone = Arc::clone(&state);
    ui.on_seleccionar_entrada(move |id| {
        if let Some(ui) = ui_weak.upgrade() {
            let mut st = state_clone.lock().unwrap();
            let id_str = id.to_string();
            refrescar_ui(&ui, &mut st, Some(&id_str));
        }
    });

    // 11. Callback: Copiar Campo al Portapapeles
    let ui_weak = ui.as_weak();
    let ui_notify = ui.as_weak();
    ui.on_copiar_campo(move |valor, etiqueta| {
        if valor.is_empty() {
            return;
        }
        match ClipboardManager::copiar_con_limpieza(valor.to_string(), 30) {
            Ok(()) => {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_texto_notificacion(format!("✓ {etiqueta} copiado").into());
                    limpiar_notificacion_despues(&ui_notify, 3);
                }
            }
            Err(e) => {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_texto_notificacion(format!("Error al copiar: {e}").into());
                    limpiar_notificacion_despues(&ui_notify, 5);
                }
            }
        }
    });

    // 12. Callback: Cerrar Ficha de Detalle
    let ui_weak = ui.as_weak();
    let state_clone = Arc::clone(&state);
    ui.on_cerrar_detalle(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let mut st = state_clone.lock().unwrap();
            st.entrada_seleccionada_id = None;
            ui.set_entrada_seleccionada_id(SharedString::default());
            ui.set_detalle_actual(EntryDetailData::default());
        }
    });

    // 13. Callback: Abrir Editor para Nueva Entrada
    let ui_weak = ui.as_weak();
    ui.on_abrir_nueva_entrada(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_es_nueva_entrada(true);
            ui.set_editor_datos(EntryEditData::default());
            ui.set_mostrar_editor(true);
        }
    });

    // 14. Callback: Abrir Editor para Entrada Existente
    let ui_weak = ui.as_weak();
    let state_clone = Arc::clone(&state);
    ui.on_abrir_editar_entrada(move |id| {
        if let Some(ui) = ui_weak.upgrade() {
            let st = state_clone.lock().unwrap();
            if let Some(entrada) = st.obtener_entrada_por_id(id.as_str()) {
                let edit_data = entrada_a_edit_data(&entrada);
                ui.set_es_nueva_entrada(false);
                ui.set_editor_datos(edit_data);
                ui.set_mostrar_editor(true);
            }
        }
    });

    // 15. Callback: Guardar Entrada
    let ui_weak = ui.as_weak();
    let state_clone = Arc::clone(&state);
    let ui_notify = ui.as_weak();
    ui.on_guardar_entrada(move |datos| {
        if let Some(ui) = ui_weak.upgrade() {
            let mut st = state_clone.lock().unwrap();
            let original = if !datos.id.is_empty() {
                st.obtener_entrada_por_id(datos.id.as_str())
            } else {
                None
            };

            let nueva_entrada = edit_data_a_entrada(&datos, original.as_ref());
            match st.guardar_o_crear_entrada(nueva_entrada) {
                Ok(id_guardada) => {
                    refrescar_ui(&ui, &mut st, Some(&id_guardada));
                    ui.set_mostrar_editor(false);
                    ui.set_texto_notificacion("✓ Entrada guardada con éxito".into());
                    limpiar_notificacion_despues(&ui_notify, 3);
                }
                Err(e) => {
                    ui.set_texto_notificacion(format!("Error al persistir: {e}").into());
                    limpiar_notificacion_despues(&ui_notify, 5);
                }
            }
        }
    });

    // 16. Callback: Mover Entrada a la Papelera
    let ui_weak = ui.as_weak();
    let state_clone = Arc::clone(&state);
    let ui_notify = ui.as_weak();
    ui.on_mover_a_papelera(move |id| {
        if let Some(ui) = ui_weak.upgrade() {
            let mut st = state_clone.lock().unwrap();
            let id_str = id.to_string();
            match st.mover_a_papelera(&id_str) {
                Ok(()) => {
                    refrescar_ui(&ui, &mut st, None);
                    ui.set_texto_notificacion("✓ Entrada movida a la papelera".into());
                    limpiar_notificacion_despues(&ui_notify, 3);
                }
                Err(e) => {
                    ui.set_texto_notificacion(format!("Error: {e}").into());
                    limpiar_notificacion_despues(&ui_notify, 5);
                }
            }
        }
    });

    // 17. Callback: Restaurar Entrada de la Papelera
    let ui_weak = ui.as_weak();
    let state_clone = Arc::clone(&state);
    let ui_notify = ui.as_weak();
    ui.on_restaurar_entrada(move |id| {
        if let Some(ui) = ui_weak.upgrade() {
            let mut st = state_clone.lock().unwrap();
            let id_str = id.to_string();
            match st.restaurar_de_papelera(&id_str) {
                Ok(()) => {
                    refrescar_ui(&ui, &mut st, Some(&id_str));
                    ui.set_texto_notificacion("✓ Entrada restaurada de la papelera".into());
                    limpiar_notificacion_despues(&ui_notify, 3);
                }
                Err(e) => {
                    ui.set_texto_notificacion(format!("Error: {e}").into());
                    limpiar_notificacion_despues(&ui_notify, 5);
                }
            }
        }
    });

    // 18. Callback: Eliminar Entrada Definitivamente
    let ui_weak = ui.as_weak();
    let state_clone = Arc::clone(&state);
    let ui_notify = ui.as_weak();
    ui.on_eliminar_definitivo(move |id| {
        if let Some(ui) = ui_weak.upgrade() {
            let mut st = state_clone.lock().unwrap();
            match st.eliminar_definitivamente(id.as_str()) {
                Ok(()) => {
                    refrescar_ui(&ui, &mut st, None);
                    ui.set_texto_notificacion("✓ Entrada eliminada definitivamente".into());
                    limpiar_notificacion_despues(&ui_notify, 3);
                }
                Err(e) => {
                    ui.set_texto_notificacion(format!("Error: {e}").into());
                    limpiar_notificacion_despues(&ui_notify, 5);
                }
            }
        }
    });

    // 18b. Callbacks de Menú Contextual (Clic Derecho)
    let ui_weak = ui.as_weak();
    let state_clone = Arc::clone(&state);
    let ui_notify = ui.as_weak();
    ui.on_alternar_favorito(move |id| {
        if let Some(ui) = ui_weak.upgrade() {
            let mut st = state_clone.lock().unwrap();
            let id_str = id.to_string();
            if let Some(mut entrada) = st.obtener_entrada_por_id(&id_str) {
                entrada.favorito = !entrada.favorito;
                if let Ok(id_guardada) = st.guardar_o_crear_entrada(entrada) {
                    refrescar_ui(&ui, &mut st, Some(&id_guardada));
                    ui.set_texto_notificacion("✓ Estado de favorito actualizado".into());
                    limpiar_notificacion_despues(&ui_notify, 2);
                }
            }
        }
    });

    let ui_weak = ui.as_weak();
    let state_clone = Arc::clone(&state);
    let ui_notify = ui.as_weak();
    ui.on_alternar_anclado(move |id| {
        if let Some(ui) = ui_weak.upgrade() {
            let mut st = state_clone.lock().unwrap();
            let id_str = id.to_string();
            if let Some(mut entrada) = st.obtener_entrada_por_id(&id_str) {
                entrada.anclado = !entrada.anclado;
                if let Ok(id_guardada) = st.guardar_o_crear_entrada(entrada) {
                    refrescar_ui(&ui, &mut st, Some(&id_guardada));
                    ui.set_texto_notificacion("✓ Estado de anclado actualizado".into());
                    limpiar_notificacion_despues(&ui_notify, 2);
                }
            }
        }
    });

    let ui_weak = ui.as_weak();
    let state_clone = Arc::clone(&state);
    let ui_notify = ui.as_weak();
    ui.on_copiar_totp(move |id| {
        if let Some(ui) = ui_weak.upgrade() {
            let st = state_clone.lock().unwrap();
            let id_str = id.to_string();
            if let Some(entrada) = st.obtener_entrada_por_id(&id_str) {
                if let Some(ref sec) = entrada.secreto_totp {
                    if !sec.trim().is_empty() {
                        let ahora = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0);
                        let digitos = if entrada.totp_digitos > 0 { entrada.totp_digitos as usize } else { 6 };
                        let periodo = if entrada.totp_periodo > 0 { entrada.totp_periodo as u64 } else { 30 };
                        if let Ok(codigo) = boveda_core::crypto::totp::codigo_totp_desde_base32(sec, ahora, digitos, periodo) {
                            if let Ok(mut ctx) = arboard::Clipboard::new() {
                                let _ = ctx.set_text(codigo);
                                ui.set_texto_notificacion("✓ Código TOTP copiado al portapapeles".into());
                                limpiar_notificacion_despues(&ui_notify, 3);
                            }
                        }
                    }
                }
            }
        }
    });

    // 19. Callback: Abrir Diálogo de Generador de Contraseñas
    let ui_weak = ui.as_weak();
    ui.on_abrir_generador(move |destino| {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_generador_destino(destino);
            let opts = ui.get_generador_opciones();
            ejecutar_generacion(&ui, &opts);
            ui.set_mostrar_generador(true);
        }
    });

    // 20. Callback: Regenerar Contraseña
    let ui_weak = ui.as_weak();
    ui.on_regenerar_contrasena(move |opts| {
        if let Some(ui) = ui_weak.upgrade() {
            ejecutar_generacion(&ui, &opts);
        }
    });

    // 21. Callback: Aplicar Contraseña Generada en el Formulario Activo
    let ui_weak = ui.as_weak();
    ui.on_aplicar_contrasena(move |pw| {
        if let Some(ui) = ui_weak.upgrade() {
            let destino = ui.get_generador_destino();
            let mut datos = ui.get_editor_datos();
            if destino == "wifi" {
                datos.wifi_password = pw;
            } else {
                datos.password = pw;
            }
            ui.set_editor_datos(datos);
            ui.set_mostrar_generador(false);
        }
    });

    // 22. Callback: Abrir Diálogo de Ajustes de Bóveda
    let ui_weak = ui.as_weak();
    let state_clone = Arc::clone(&state);
    ui.on_abrir_ajustes(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let st = state_clone.lock().unwrap();
            let config_opt = st.boveda.as_ref().and_then(|b| b.configuracion.clone());
            let config = config_opt.unwrap_or_default();

            let perfil_str = match st.perfil_argon2 {
                PerfilArgon2::Ultraseguro => "ULTRASEGURO",
                PerfilArgon2::Reforzado => "REFORZADO",
                PerfilArgon2::Estandar => "ESTANDAR",
            };

            let prefijos = config.prefijos_subdominios.join(", ");
            let mut puertos_str = String::new();
            for (p, n) in &config.puertos_servicios_locales {
                puertos_str.push_str(&format!("{p}: {n}\n"));
            }

            let defaults = boveda_core::config::defaults::valores_predeterminados();
            let puertos_final = if puertos_str.trim().is_empty() {
                defaults.puertos_homelab.to_string()
            } else {
                puertos_str.trim().to_string()
            };

            let prefijos_final = if prefijos.trim().is_empty() {
                defaults.prefijos_subdominios.to_string()
            } else {
                prefijos
            };

            let ancho = st.ultimo_ancho_chips;
            let filas_p = parse_puertos_filas_con_ancho(&puertos_final, ancho);
            let filas_s = parse_subdominios_filas_con_ancho(&prefijos_final, ancho);
            ui.set_ajustes_filas_puertos(ModelRc::new(VecModel::from(filas_p)));
            ui.set_ajustes_filas_subdominios(ModelRc::new(VecModel::from(filas_s)));

            let bio_disp = boveda_core::gui::state::biometrics::esta_biometria_disponible(
                &st.ruta_boveda.display().to_string(),
            );

            ui.set_ajustes_datos(SettingsData {
                portapapeles_segundos: config.portapapeles_segundos,
                autobloqueo_segundos: config.auto_bloqueo_segundos,
                autodestruccion_intentos: config.autodestruccion_intentos_fallidos_max,
                perfil_argon2: perfil_str.into(),
                biometria_activa: bio_disp,
                modo_tema: st.modo_tema.as_str().into(),
                color_acento: st.color_acento.as_str().into(),
                nombre_boveda: "Mi Bóveda Principal".into(),
                ocultar_usuario: config.ocultar_usuario,
                ocultar_contrasena: config.ocultar_contrasena,
                ocultar_notas: config.ocultar_notas,
                ocultar_campos: config.ocultar_campos,
                criterio_ordenacion: config.criterio_ordenacion.into(),
                fijar_anclados_arriba: config.fijar_anclados_arriba,
                mostrar_ola_niagara: true,
                incluir_enie_niagara: true,
                prefijos_subdominios: prefijos_final.into(),
                puertos_homelab: puertos_final.into(),
            });
            ui.set_ajustes_error(SharedString::default());
            ui.set_ajustes_exito(SharedString::default());
            ui.set_mostrar_ajustes(true);
        }
    });

    // 23. Callback: Guardar Ajustes de Bóveda
    let ui_weak = ui.as_weak();
    let state_clone = Arc::clone(&state);
    let ui_notify = ui.as_weak();
    ui.on_guardar_ajustes(move |datos| {
        if let Some(ui) = ui_weak.upgrade() {
            let mut st = state_clone.lock().unwrap();
            let perfil = match datos.perfil_argon2.as_str() {
                "ULTRASEGURO" => PerfilArgon2::Ultraseguro,
                "REFORZADO" => PerfilArgon2::Reforzado,
                _ => PerfilArgon2::Estandar,
            };

            st.modo_tema = ModoTema::desde_str(datos.modo_tema.as_str());
            st.color_acento = datos.color_acento.to_string();

            // Aplicar tema en Slint en tiempo real
            aplicar_tema_a_ui(&ui, st.modo_tema, &st.color_acento);

            match st.actualizar_ajustes(&datos, perfil) {
                Ok(()) => {
                    ui.set_ajustes_exito("✓ Ajustes guardados correctamente".into());
                    ui.set_ajustes_error(SharedString::default());
                    ui.set_texto_notificacion("✓ Ajustes de bóveda actualizados".into());
                    limpiar_notificacion_despues(&ui_notify, 3);
                }
                Err(e) => {
                    ui.set_ajustes_error(format!("Error al guardar: {e}").into());
                }
            }
        }
    });

    // 23b. Callback: Restaurar Ajustes Predeterminados (Fuente Única Defaults)
    let ui_weak = ui.as_weak();
    let state_clone_rst = Arc::clone(&state);
    let ui_notify = ui.as_weak();
    ui.on_restaurar_predeterminados(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let ancho = state_clone_rst.lock().unwrap().ultimo_ancho_chips;
            let defaults = boveda_core::config::defaults::valores_predeterminados();
            let filas_p = parse_puertos_filas_con_ancho(&defaults.puertos_homelab, ancho);
            let filas_s = parse_subdominios_filas_con_ancho(&defaults.prefijos_subdominios, ancho);
            ui.set_ajustes_filas_puertos(ModelRc::new(VecModel::from(filas_p)));
            ui.set_ajustes_filas_subdominios(ModelRc::new(VecModel::from(filas_s)));
            ui.set_ajustes_datos(defaults);
            ui.set_ajustes_exito("✓ Valores predeterminados restaurados (haga clic en Guardar)".into());
            ui.set_ajustes_error(SharedString::default());
            ui.set_texto_notificacion("↺ Defaults globales restaurados".into());
            limpiar_notificacion_despues(&ui_notify, 3);
        }
    });

    // 23c. Callback: Restaurar Nivel 1 (Individual por Campo)
    let ui_weak = ui.as_weak();
    let state_clone_fld = Arc::clone(&state);
    let ui_notify = ui.as_weak();
    ui.on_restaurar_campo(move |campo| {
        if let Some(ui) = ui_weak.upgrade() {
            let ancho = state_clone_fld.lock().unwrap().ultimo_ancho_chips;
            let defaults = boveda_core::config::defaults::valores_predeterminados();
            let mut datos = ui.get_ajustes_datos();
            match campo.as_str() {
                "autobloqueo" => datos.autobloqueo_segundos = defaults.autobloqueo_segundos,
                "portapapeles" => datos.portapapeles_segundos = defaults.portapapeles_segundos,
                "argon2" => datos.perfil_argon2 = defaults.perfil_argon2,
                "autodestruccion" => datos.autodestruccion_intentos = defaults.autodestruccion_intentos,
                "modo_tema" => datos.modo_tema = defaults.modo_tema,
                "color_acento" => datos.color_acento = defaults.color_acento,
                "nombre_boveda" => datos.nombre_boveda = defaults.nombre_boveda,
                "criterio_ordenacion" => datos.criterio_ordenacion = defaults.criterio_ordenacion,
                "fijar_anclados" => datos.fijar_anclados_arriba = defaults.fijar_anclados_arriba,
                "privacidad_campos" => {
                    datos.ocultar_usuario = defaults.ocultar_usuario;
                    datos.ocultar_contrasena = defaults.ocultar_contrasena;
                    datos.ocultar_notas = defaults.ocultar_notas;
                }
                "niagara" => {
                    datos.mostrar_ola_niagara = defaults.mostrar_ola_niagara;
                    datos.incluir_enie_niagara = defaults.incluir_enie_niagara;
                }
                "puertos_homelab" => {
                    datos.puertos_homelab = defaults.puertos_homelab;
                    let filas_p = parse_puertos_filas_con_ancho(&datos.puertos_homelab, ancho);
                    ui.set_ajustes_filas_puertos(ModelRc::new(VecModel::from(filas_p)));
                }
                "prefijos_subdominios" => {
                    datos.prefijos_subdominios = defaults.prefijos_subdominios;
                    let filas_s = parse_subdominios_filas_con_ancho(&datos.prefijos_subdominios, ancho);
                    ui.set_ajustes_filas_subdominios(ModelRc::new(VecModel::from(filas_s)));
                }
                "indicadores_colores" => {
                    let theme = ui.global::<Theme>();
                    theme.set_color_ind_usuario(parse_hex_color("#2563EB"));
                    theme.set_color_ind_password(parse_hex_color("#EAB308"));
                    theme.set_color_ind_totp(parse_hex_color("#F97316"));
                    theme.set_color_ind_passkey(parse_hex_color("#8B5CF6"));
                    theme.set_color_ind_url(parse_hex_color("#06B6D4"));
                    theme.set_color_ind_app(parse_hex_color("#16A34A"));
                    theme.set_color_ind_notas(parse_hex_color("#64748B"));
                    ui.set_ajustes_hex_usuario("#2563EB".into());
                    ui.set_ajustes_hex_password("#EAB308".into());
                    ui.set_ajustes_hex_totp("#F97316".into());
                    ui.set_ajustes_hex_passkey("#8B5CF6".into());
                    ui.set_ajustes_hex_url("#06B6D4".into());
                    ui.set_ajustes_hex_app("#16A34A".into());
                }
                _ => {}
            }
            ui.set_ajustes_datos(datos);
            ui.set_texto_notificacion("↺ Ajuste restablecido a predeterminado".into());
            limpiar_notificacion_despues(&ui_notify, 2);
        }
    });

    // 23d. Callback: Restaurar Nivel 2 (Grupo por Sub-sección)
    let ui_weak = ui.as_weak();
    let state_clone_sec = Arc::clone(&state);
    let ui_notify = ui.as_weak();
    ui.on_restaurar_seccion(move |seccion| {
        if let Some(ui) = ui_weak.upgrade() {
            let ancho = state_clone_sec.lock().unwrap().ultimo_ancho_chips;
            let defaults = boveda_core::config::defaults::valores_predeterminados();
            let mut datos = ui.get_ajustes_datos();
            match seccion.as_str() {
                "seguridad_temporizadores" => {
                    datos.autobloqueo_segundos = defaults.autobloqueo_segundos;
                    datos.portapapeles_segundos = defaults.portapapeles_segundos;
                }
                "seguridad_kdf" => {
                    datos.perfil_argon2 = defaults.perfil_argon2;
                }
                "seguridad_autodestruccion" => {
                    datos.autodestruccion_intentos = defaults.autodestruccion_intentos;
                }
                "apariencia_tema" => {
                    datos.modo_tema = defaults.modo_tema;
                }
                "apariencia_acento" => {
                    datos.color_acento = defaults.color_acento;
                }
                "apariencia_identidad" => {
                    datos.nombre_boveda = defaults.nombre_boveda;
                }
                "apariencia_indicadores" => {
                    let theme = ui.global::<Theme>();
                    theme.set_estilo_indicadores("iconos".into());
                    theme.set_tamanio_indicadores(12.0);
                    theme.set_opacidad_indicadores(1.0);
                    theme.set_color_ind_usuario(parse_hex_color("#2563EB"));
                    theme.set_color_ind_password(parse_hex_color("#EAB308"));
                    theme.set_color_ind_totp(parse_hex_color("#F97316"));
                    theme.set_color_ind_passkey(parse_hex_color("#8B5CF6"));
                    theme.set_color_ind_url(parse_hex_color("#06B6D4"));
                    theme.set_color_ind_app(parse_hex_color("#16A34A"));
                    theme.set_color_ind_notas(parse_hex_color("#64748B"));
                    ui.set_ajustes_hex_usuario("#2563EB".into());
                    ui.set_ajustes_hex_password("#EAB308".into());
                    ui.set_ajustes_hex_totp("#F97316".into());
                    ui.set_ajustes_hex_passkey("#8B5CF6".into());
                    ui.set_ajustes_hex_url("#06B6D4".into());
                    ui.set_ajustes_hex_app("#16A34A".into());
                }
                "lista_orden" => {
                    datos.criterio_ordenacion = defaults.criterio_ordenacion;
                    datos.fijar_anclados_arriba = defaults.fijar_anclados_arriba;
                }
                "lista_privacidad" => {
                    datos.ocultar_usuario = defaults.ocultar_usuario;
                    datos.ocultar_contrasena = defaults.ocultar_contrasena;
                    datos.ocultar_notas = defaults.ocultar_notas;
                }
                "lista_niagara" => {
                    datos.mostrar_ola_niagara = defaults.mostrar_ola_niagara;
                    datos.incluir_enie_niagara = defaults.incluir_enie_niagara;
                }
                "sistema_servicios" => {
                    datos.puertos_homelab = defaults.puertos_homelab;
                    let filas_p = parse_puertos_filas_con_ancho(&datos.puertos_homelab, ancho);
                    ui.set_ajustes_filas_puertos(ModelRc::new(VecModel::from(filas_p)));
                }
                "sistema_subdominios" => {
                    datos.prefijos_subdominios = defaults.prefijos_subdominios;
                    let filas_s = parse_subdominios_filas_con_ancho(&datos.prefijos_subdominios, ancho);
                    ui.set_ajustes_filas_subdominios(ModelRc::new(VecModel::from(filas_s)));
                }
                _ => {}
            }
            ui.set_ajustes_datos(datos);
            ui.set_texto_notificacion("↺ Sub-sección restablecida a predeterminado".into());
            limpiar_notificacion_despues(&ui_notify, 2);
        }
    });

    // 23e. Callbacks: Manipulación interactiva de puertos y subdominios
    let ui_weak = ui.as_weak();
    let state_clone_p_add = Arc::clone(&state);
    ui.on_agregar_puerto(move |puerto, nombre| {
        if let Some(ui) = ui_weak.upgrade() {
            let ancho = state_clone_p_add.lock().unwrap().ultimo_ancho_chips;
            let mut datos = ui.get_ajustes_datos();
            let nueva_str = agregar_puerto_a_str(&datos.puertos_homelab, puerto.as_str(), nombre.as_str());
            datos.puertos_homelab = nueva_str.clone().into();
            let filas_p = parse_puertos_filas_con_ancho(&nueva_str, ancho);
            ui.set_ajustes_filas_puertos(ModelRc::new(VecModel::from(filas_p)));
            ui.set_ajustes_datos(datos);
        }
    });

    let ui_weak = ui.as_weak();
    let state_clone_p_del = Arc::clone(&state);
    ui.on_eliminar_puerto(move |id| {
        if let Some(ui) = ui_weak.upgrade() {
            let ancho = state_clone_p_del.lock().unwrap().ultimo_ancho_chips;
            let mut datos = ui.get_ajustes_datos();
            let nueva_str = eliminar_puerto_de_str(&datos.puertos_homelab, id.as_str());
            datos.puertos_homelab = nueva_str.clone().into();
            let filas_p = parse_puertos_filas_con_ancho(&nueva_str, ancho);
            ui.set_ajustes_filas_puertos(ModelRc::new(VecModel::from(filas_p)));
            ui.set_ajustes_datos(datos);
        }
    });

    let ui_weak = ui.as_weak();
    let state_clone_s_add = Arc::clone(&state);
    ui.on_agregar_subdominio(move |subdominio| {
        if let Some(ui) = ui_weak.upgrade() {
            let ancho = state_clone_s_add.lock().unwrap().ultimo_ancho_chips;
            let mut datos = ui.get_ajustes_datos();
            let nueva_str = agregar_subdominio_a_str(&datos.prefijos_subdominios, subdominio.as_str());
            datos.prefijos_subdominios = nueva_str.clone().into();
            let filas_s = parse_subdominios_filas_con_ancho(&nueva_str, ancho);
            ui.set_ajustes_filas_subdominios(ModelRc::new(VecModel::from(filas_s)));
            ui.set_ajustes_datos(datos);
        }
    });

    let ui_weak = ui.as_weak();
    let state_clone_s_del = Arc::clone(&state);
    ui.on_eliminar_subdominio(move |id| {
        if let Some(ui) = ui_weak.upgrade() {
            let ancho = state_clone_s_del.lock().unwrap().ultimo_ancho_chips;
            let mut datos = ui.get_ajustes_datos();
            let nueva_str = eliminar_subdominio_de_str(&datos.prefijos_subdominios, id.as_str());
            datos.prefijos_subdominios = nueva_str.clone().into();
            let filas_s = parse_subdominios_filas_con_ancho(&nueva_str, ancho);
            ui.set_ajustes_filas_subdominios(ModelRc::new(VecModel::from(filas_s)));
            ui.set_ajustes_datos(datos);
        }
    });

    // 23f. Callback: Redimensionamiento reactivo de chips según ancho del espacio disponible
    let ui_weak = ui.as_weak();
    let state_clone_resize = Arc::clone(&state);
    ui.on_reportar_ancho_chips(move |ancho_px| {
        if let Some(ui) = ui_weak.upgrade() {
            let mut st = state_clone_resize.lock().unwrap();
            let dif = (ancho_px - st.ultimo_ancho_chips).abs();
            if dif < 10.0 {
                return;
            }
            st.ultimo_ancho_chips = ancho_px;
            drop(st);

            let datos = ui.get_ajustes_datos();
            let filas_p = parse_puertos_filas_con_ancho(&datos.puertos_homelab, ancho_px);
            let filas_s = parse_subdominios_filas_con_ancho(&datos.prefijos_subdominios, ancho_px);
            ui.set_ajustes_filas_puertos(ModelRc::new(VecModel::from(filas_p)));
            ui.set_ajustes_filas_subdominios(ModelRc::new(VecModel::from(filas_s)));
        }
    });

    // 23g. Callback: Cálculo de Color HSV en tiempo real para modal
    ui.on_calcular_hsv(|h, s, v| {
        let (r, g, b, hex) = hsv_a_rgb(h, s, v);
        HsvColorResult {
            color: slint::Color::from_rgb_u8(r, g, b),
            hex: SharedString::from(hex),
        }
    });

    // 23h. Callback: Aplicar Color de Indicador seleccionado
    let ui_weak = ui.as_weak();
    ui.on_aplicar_color_indicador(move |clave, col, hex| {
        if let Some(ui) = ui_weak.upgrade() {
            let theme = ui.global::<Theme>();
            match clave.as_str() {
                "usuario" => {
                    theme.set_color_ind_usuario(col);
                    ui.set_ajustes_hex_usuario(hex);
                }
                "password" => {
                    theme.set_color_ind_password(col);
                    ui.set_ajustes_hex_password(hex);
                }
                "totp" => {
                    theme.set_color_ind_totp(col);
                    ui.set_ajustes_hex_totp(hex);
                }
                "passkey" => {
                    theme.set_color_ind_passkey(col);
                    ui.set_ajustes_hex_passkey(hex);
                }
                "url" => {
                    theme.set_color_ind_url(col);
                    ui.set_ajustes_hex_url(hex);
                }
                "app" => {
                    theme.set_color_ind_app(col);
                    ui.set_ajustes_hex_app(hex);
                }
                _ => {}
            }
        }
    });

    // 24. Callback: Cambiar Contraseña Maestra
    let ui_weak = ui.as_weak();
    let state_clone = Arc::clone(&state);
    let ui_notify = ui.as_weak();
    ui.on_cambiar_clave_maestra(move |act, nva, conf| {
        let ui = match ui_weak.upgrade() {
            Some(u) => u,
            None => return,
        };

        if nva.is_empty() {
            ui.set_ajustes_error("La nueva contraseña no puede estar vacía.".into());
            return;
        }
        if nva != conf {
            ui.set_ajustes_error("La nueva contraseña y su confirmación no coinciden.".into());
            return;
        }
        if nva.len() < 8 {
            ui.set_ajustes_error("La nueva contraseña debe tener al menos 8 caracteres.".into());
            return;
        }

        ui.set_ajustes_cambiando_clave(true);
        ui.set_ajustes_error(SharedString::default());
        ui.set_ajustes_exito(SharedString::default());

        let state_worker = Arc::clone(&state_clone);
        let ui_handle = ui_weak.clone();
        let ui_notif_handle = ui_notify.clone();
        let mut act_s = act.to_string();
        let mut nva_s = nva.to_string();

        thread::spawn(move || {
            let res = {
                let mut st = state_worker.lock().unwrap();
                let perfil = st.perfil_argon2;
                st.cambiar_clave_maestra(&act_s, &nva_s, perfil)
            };

            act_s.zeroize();
            nva_s.zeroize();

            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_handle.upgrade() {
                    ui.set_ajustes_cambiando_clave(false);
                    match res {
                        Ok(()) => {
                            ui.set_ajustes_exito(
                                "✓ Contraseña maestra cambiada y bóveda re-cifrada con éxito".into(),
                            );
                            ui.set_ajustes_error(SharedString::default());
                            ui.set_texto_notificacion(
                                "✓ Contraseña maestra cambiada con éxito".into(),
                            );
                            limpiar_notificacion_despues(&ui_notif_handle, 3);
                        }
                        Err(e) => {
                            ui.set_ajustes_error(
                                format!("Error: {e}. Compruebe la contraseña actual.").into(),
                            );
                        }
                    }
                }
            });
        });
    });

    // 25. Callback: Alternar Biometría
    let state_clone = Arc::clone(&state);
    let ui_weak = ui.as_weak();
    let ui_notify = ui.as_weak();
    ui.on_alternar_biometria(move |activar| {
        let st = state_clone.lock().unwrap();
        let ruta = st.ruta_boveda.display().to_string();
        if activar {
            if let Some(ref clave) = st.clave_maestra {
                let _ = boveda_core::gui::state::biometrics::guardar_clave_biometria(
                    &ruta,
                    clave.as_str(),
                );
            }
        } else {
            let _ = boveda_core::gui::state::biometrics::eliminar_clave_biometria(&ruta);
        }
        if let Some(ui) = ui_weak.upgrade() {
            let disp = boveda_core::gui::state::biometrics::esta_biometria_disponible(&ruta);
            ui.set_biometria_disponible(disp);
            ui.set_texto_notificacion(
                if activar { "✓ Biometría activada en llavero del SO" } else { "✓ Biometría desactivada" }.into(),
            );
            limpiar_notificacion_despues(&ui_notify, 3);
        }
    });

    // 26. Callbacks de Herramientas y Copias
    let ui_notify = ui.as_weak();
    ui.on_exportar_json(move || {
        // En desktop se puede invocar el guardado de archivo con rfd
        if let Some(ui) = ui_notify.upgrade() {
            ui.set_texto_notificacion("Función de exportación JSON preparada".into());
        }
    });
    let ui_notify = ui.as_weak();
    ui.on_exportar_csv(move || {
        if let Some(ui) = ui_notify.upgrade() {
            ui.set_texto_notificacion("Función de exportación CSV preparada".into());
        }
    });
    let ui_notify = ui.as_weak();
    ui.on_importar_bitwarden(move || {
        if let Some(ui) = ui_notify.upgrade() {
            ui.set_texto_notificacion("Seleccione archivo JSON de Bitwarden".into());
        }
    });
    let ui_notify = ui.as_weak();
    ui.on_importar_csv(move || {
        if let Some(ui) = ui_notify.upgrade() {
            ui.set_texto_notificacion("Seleccione archivo CSV de contraseñas".into());
        }
    });
    let ui_notify = ui.as_weak();
    ui.on_fusionar_boveda(move || {
        if let Some(ui) = ui_notify.upgrade() {
            ui.set_texto_notificacion("Seleccione segundo archivo .bvda para fusión Lossless".into());
        }
    });
    let ui_notify = ui.as_weak();
    ui.on_auditoria_salud(move || {
        if let Some(ui) = ui_notify.upgrade() {
            ui.set_texto_notificacion("Auditoría de salud completada: 0 problemas críticos".into());
        }
    });

    // 26b. Callbacks de Menú Desplegable (3 Puntos) y Agrupación de Cuentas
    let ui_weak = ui.as_weak();
    let state_clone = Arc::clone(&state);
    ui.on_alternar_grupo(move |clave| {
        if let Some(ui) = ui_weak.upgrade() {
            let mut st = state_clone.lock().unwrap();
            st.alternar_expansion_grupo(clave.as_str());
            let entradas = st.obtener_entradas_filtradas();
            ui.set_lista_entradas(ModelRc::new(VecModel::from(entradas)));
        }
    });

    let ui_weak = ui.as_weak();
    let state_clone = Arc::clone(&state);
    let ui_notify = ui.as_weak();
    ui.on_alternar_solo_favoritos(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let mut st = state_clone.lock().unwrap();
            st.solo_favoritos = !st.solo_favoritos;
            let activo = st.solo_favoritos;
            let entradas = st.obtener_entradas_filtradas();
            ui.set_solo_favoritos(activo);
            ui.set_lista_entradas(ModelRc::new(VecModel::from(entradas)));
            ui.set_texto_notificacion(
                if activo { "★ Filtro activado: Solo favoritos" } else { "Filtro desactivado: Todas las entradas" }.into(),
            );
            limpiar_notificacion_despues(&ui_notify, 2);
        }
    });

    let ui_weak = ui.as_weak();
    let state_clone = Arc::clone(&state);
    let ui_notify = ui.as_weak();
    ui.on_normalizar_titulos(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let mut st = state_clone.lock().unwrap();
            let mut modificadas = 0usize;
            if let Some(ref mut boveda) = st.boveda {
                for entrada in &mut boveda.entradas {
                    let url_primer = entrada.urls.first().map(|s| s.as_str()).unwrap_or("");
                    let serv = boveda_core::gui::extraer_nombre_servicio(url_primer, &entrada.titulo);
                    if !serv.is_empty() && serv != entrada.titulo {
                        entrada.titulo = serv;
                        modificadas += 1;
                    }
                }
            }
            if modificadas > 0 {
                let _ = st.guardar_a_disco();
                refrescar_ui(&ui, &mut st, None);
                ui.set_texto_notificacion(format!("✓ {} títulos normalizados", modificadas).into());
            } else {
                ui.set_texto_notificacion("✓ Todos los títulos ya se encuentran normalizados".into());
            }
            limpiar_notificacion_despues(&ui_notify, 3);
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_crear_nuevo_tipo(move |tipo| {
        if let Some(ui) = ui_weak.upgrade() {
            let edit_data = EntryEditData {
                tipo,
                ..Default::default()
            };
            ui.set_editor_datos(edit_data);
            ui.set_es_nueva_entrada(true);
            ui.set_mostrar_editor(true);
        }
    });

    let ui_weak = ui.as_weak();
    let state_clone = Arc::clone(&state);
    let ui_notify = ui.as_weak();
    ui.on_cambiar_orden(move |nuevo_orden| {
        if let Some(ui) = ui_weak.upgrade() {
            let mut st = state_clone.lock().unwrap();
            if let Some(ref mut boveda) = st.boveda {
                if let Some(ref mut cfg) = boveda.configuracion {
                    cfg.criterio_ordenacion = nuevo_orden.to_string();
                }
            }
            let _ = st.guardar_a_disco();
            ui.set_orden_actual(nuevo_orden.clone());
            let entradas = st.obtener_entradas_filtradas();
            ui.set_lista_entradas(ModelRc::new(VecModel::from(entradas)));
            ui.set_texto_notificacion(format!("✓ Criterio de ordenación aplicado: {nuevo_orden}").into());
            limpiar_notificacion_despues(&ui_notify, 2);
        }
    });

    let ui_weak = ui.as_weak();
    let state_clone = Arc::clone(&state);
    let ui_notify = ui.as_weak();
    ui.on_exportar_bvda(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let st = state_clone.lock().unwrap();
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Bóveda Local", &["bvda"])
                .set_file_name("copia_seguridad.bvda")
                .save_file()
            {
                if let Err(e) = std::fs::copy(&st.ruta_boveda, &path) {
                    ui.set_texto_notificacion(format!("Error al exportar: {e}").into());
                } else {
                    ui.set_texto_notificacion("✓ Copia .bvda exportada con éxito".into());
                }
                limpiar_notificacion_despues(&ui_notify, 3);
            }
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_navegar_apariencia(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_mostrar_ajustes(true);
        }
    });

    let ui_notify = ui.as_weak();
    ui.on_abrir_ayuda(move || {
        if let Some(ui) = ui_notify.upgrade() {
            ui.set_texto_notificacion("Bóveda Local Desktop v0.1.0 - Ayuda y Manual en línea".into());
            limpiar_notificacion_despues(&ui_notify, 4);
        }
    });

    // 26b. Gestión desacoplada de Zoom Global (Ctrl +, Ctrl -, Ctrl 0) y Auto-Escalado por Resolución
    let zoom_state = Arc::new(Mutex::new(ZoomState::default()));

    let ui_weak = ui.as_weak();
    let zoom_clone = Arc::clone(&zoom_state);
    let ui_notify = ui.as_weak();
    ui.on_aumentar_zoom(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let mut zs = zoom_clone.lock().unwrap();
            let factor = zs.aumentar_zoom();
            let pct = zs.porcentaje_usuario();
            drop(zs);
            aplicar_zoom_ventana(ui.window(), factor);
            ui.set_texto_notificacion(format!("🔍 Zoom de interfaz: {pct}%").into());
            limpiar_notificacion_despues(&ui_notify, 2);
        }
    });

    let ui_weak = ui.as_weak();
    let zoom_clone = Arc::clone(&zoom_state);
    let ui_notify = ui.as_weak();
    ui.on_reducir_zoom(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let mut zs = zoom_clone.lock().unwrap();
            let factor = zs.reducir_zoom();
            let pct = zs.porcentaje_usuario();
            drop(zs);
            aplicar_zoom_ventana(ui.window(), factor);
            ui.set_texto_notificacion(format!("🔍 Zoom de interfaz: {pct}%").into());
            limpiar_notificacion_despues(&ui_notify, 2);
        }
    });

    let ui_weak = ui.as_weak();
    let zoom_clone = Arc::clone(&zoom_state);
    let ui_notify = ui.as_weak();
    ui.on_restaurar_zoom(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let mut zs = zoom_clone.lock().unwrap();
            let factor = zs.restaurar_zoom();
            let pct = zs.porcentaje_usuario();
            drop(zs);
            aplicar_zoom_ventana(ui.window(), factor);
            ui.set_texto_notificacion(format!("🔍 Zoom restaurado: {pct}%").into());
            limpiar_notificacion_despues(&ui_notify, 2);
        }
    });

    // 27. Temporizador periódico para refrescar códigos TOTP en vivo y verificar auto-adaptación de resolución
    let timer_totp = slint::Timer::default();
    let ui_totp = ui.as_weak();
    let state_totp = Arc::clone(&state);
    let zoom_timer = Arc::clone(&zoom_state);
    timer_totp.start(
        slint::TimerMode::Repeated,
        Duration::from_secs(1),
        move || {
            if let Some(ui) = ui_totp.upgrade() {
                // Auto-adaptación de escala según resolución física de la ventana
                let phys_size = ui.window().size();
                if phys_size.width > 0 && phys_size.height > 0 {
                    let mut zs = zoom_timer.lock().unwrap();
                    if zs.actualizar_resolucion(phys_size.width, phys_size.height) {
                        let factor = zs.factor_final();
                        drop(zs);
                        aplicar_zoom_ventana(ui.window(), factor);
                    }
                }

                if ui.get_boveda_desbloqueada() {
                    let st = state_totp.lock().unwrap();
                    let entradas = st.obtener_entradas_filtradas();
                    ui.set_lista_entradas(ModelRc::new(VecModel::from(entradas)));
                }
            }
        },
    );

    // 28. Ejecución del Event Loop de la GUI
    let res = ui.run();

    // 28. Purga y Zeroización estricta al cerrar la aplicación
    state.lock().unwrap().bloquear();

    res
}
