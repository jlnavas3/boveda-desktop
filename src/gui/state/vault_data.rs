/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Estado central en memoria de la bóveda abierta en la interfaz gráfica (Desktop).
//! Implementa persistencia atómica directa contra archivos `.bvda` en disco.

use std::collections::HashSet;
use std::path::PathBuf;
use zeroize::Zeroizing;

use crate::crypto::{empaquetar_boveda, escribir_atomico, BvdaError, PerfilArgon2};
use crate::gui::service_grouper::aplicar_agrupamiento_servicios;
use crate::gui::state::filter_engine::{
    calcular_opciones_categorias_dropdown, calcular_opciones_tipos_dropdown,
    calcular_resumen_categorias, calcular_resumen_etiquetas, calcular_resumen_perfiles,
    calcular_resumen_tipos, construir_detalle_entrada, encontrar_id_por_letra,
    encontrar_indice_por_letra, filtrar_entradas,
};
use crate::gui::state::selection::EstadoSeleccionLista;
use crate::gui::state::theme_manager::ModoTema;
use crate::gui::{DropdownOption, EntryDetailData, EntryOverview, FilterItem, SettingsData};
use crate::models::{ContenidoBoveda, Entrada};

#[derive(Debug)]
pub struct GuiVaultState {
    pub boveda: Option<ContenidoBoveda>,
    pub ruta_boveda: PathBuf,
    pub clave_maestra: Option<Zeroizing<String>>,
    pub perfil_argon2: PerfilArgon2,
    pub filtro_tipo: String,
    pub filtro_categoria: String,
    pub filtro_perfil: String,
    pub filtro_etiqueta: String,
    pub termino_busqueda: String,
    pub entrada_seleccionada_id: Option<String>,
    pub modo_tema: ModoTema,
    pub color_acento: String,
    pub ultimo_ancho_chips: f32,
    pub solo_favoritos: bool,
    pub grupos_expandidos: HashSet<String>,
    pub seleccion: EstadoSeleccionLista,
    pub historial_generador: Vec<crate::gui::GeneratedPasswordItem>,
    pub bitacora_eventos: Vec<String>,
    pub ultima_actividad_seg: u64,
    pub intentos_fallidos_desbloqueo: i32,
}

impl GuiVaultState {
    pub fn new(ruta_boveda: PathBuf) -> Self {
        let ahora_seg = chrono::Utc::now().timestamp() as u64;
        Self {
            boveda: None,
            ruta_boveda,
            clave_maestra: None,
            perfil_argon2: PerfilArgon2::Estandar,
            filtro_tipo: "TODOS".to_string(),
            filtro_categoria: "TODAS".to_string(),
            filtro_perfil: "TODOS".to_string(),
            filtro_etiqueta: "TODAS".to_string(),
            termino_busqueda: String::new(),
            entrada_seleccionada_id: None,
            modo_tema: ModoTema::Oscuro,
            color_acento: "menta".to_string(),
            ultimo_ancho_chips: 720.0,
            solo_favoritos: false,
            grupos_expandidos: HashSet::new(),
            seleccion: EstadoSeleccionLista::new(),
            historial_generador: Vec::new(),
            bitacora_eventos: Vec::new(),
            ultima_actividad_seg: ahora_seg,
            intentos_fallidos_desbloqueo: 0,
        }
    }

    pub fn tocar_actividad(&mut self) {
        self.ultima_actividad_seg = chrono::Utc::now().timestamp() as u64;
    }

    pub fn registrar_evento(&mut self, descripcion: &str) {
        let hora = chrono::Local::now().format("%H:%M:%S").to_string();
        self.bitacora_eventos.insert(0, format!("[{hora}] {descripcion}"));
        if self.bitacora_eventos.len() > 60 {
            self.bitacora_eventos.truncate(60);
        }
        self.tocar_actividad();
    }

    pub fn cargar_boveda(
        &mut self,
        boveda: ContenidoBoveda,
        ruta: PathBuf,
        clave: Option<Zeroizing<String>>,
        perfil: Option<PerfilArgon2>,
    ) {
        self.ruta_boveda = ruta;
        self.clave_maestra = clave;
        if let Some(p) = perfil {
            self.perfil_argon2 = p;
        }
        self.boveda = Some(boveda);
        self.filtro_tipo = "TODOS".to_string();
        self.filtro_categoria = "TODAS".to_string();
        self.termino_busqueda.clear();
        self.seleccion.limpiar();
        self.intentos_fallidos_desbloqueo = 0;
        self.registrar_evento("Bóveda descifrada y verificada con éxito (AES-256-GCM + Argon2id).");
    }

    pub fn esta_desbloqueada(&self) -> bool {
        self.boveda.is_some() && self.clave_maestra.is_some()
    }

    pub fn bloquear(&mut self) {
        self.boveda = None;
        self.clave_maestra = None;
        self.filtro_tipo = "TODOS".to_string();
        self.filtro_categoria = "TODAS".to_string();
        self.termino_busqueda.clear();
        self.seleccion.limpiar();
        self.historial_generador.clear();
        self.bitacora_eventos.clear();
    }

    pub fn guardar_a_disco(&mut self) -> Result<(), BvdaError> {
        let boveda = self.boveda.as_ref().ok_or(BvdaError::AutenticacionFallida)?;
        let clave = self.clave_maestra.as_ref().ok_or(BvdaError::AutenticacionFallida)?;

        let bytes = empaquetar_boveda(
            boveda,
            clave.as_bytes(),
            self.perfil_argon2,
        )?;
        escribir_atomico(&self.ruta_boveda, &bytes)
    }

    pub fn guardar_o_crear_entrada(&mut self, mut nueva: Entrada) -> Result<String, BvdaError> {
        let boveda = self.boveda.as_mut().ok_or(BvdaError::AutenticacionFallida)?;
        let ahora = chrono::Utc::now().timestamp_millis();
        nueva.modificada_en = ahora;

        if nueva.id.is_empty() {
            nueva.id = uuid::Uuid::new_v4().to_string();
            nueva.creada_en = ahora;
        }

        let id_asignado = nueva.id.clone();
        if let Some(idx) = boveda.entradas.iter().position(|e| e.id == id_asignado) {
            boveda.entradas[idx] = nueva;
        } else {
            boveda.entradas.push(nueva);
        }

        self.guardar_a_disco()?;
        Ok(id_asignado)
    }

    pub fn mover_a_papelera(&mut self, id: &str) -> Result<(), BvdaError> {
        let boveda = self.boveda.as_mut().ok_or(BvdaError::AutenticacionFallida)?;
        let ahora = chrono::Utc::now().timestamp_millis();

        if let Some(pos) = boveda.entradas.iter().position(|e| e.id == id) {
            let mut entrada = boveda.entradas.remove(pos);
            entrada.eliminada_en = ahora;
            entrada.modificada_en = ahora;
            entrada.revision += 1;
            boveda.papelera.push(entrada);
            self.guardar_a_disco()
        } else if let Some(entrada) = boveda.papelera.iter_mut().find(|e| e.id == id) {
            entrada.eliminada_en = ahora;
            entrada.modificada_en = ahora;
            entrada.revision += 1;
            self.guardar_a_disco()
        } else {
            Err(BvdaError::RegistroNoEncontrado(format!("No existe entrada con ID '{id}'")))
        }
    }

    pub fn restaurar_de_papelera(&mut self, id: &str) -> Result<(), BvdaError> {
        let boveda = self.boveda.as_mut().ok_or(BvdaError::AutenticacionFallida)?;
        let ahora = chrono::Utc::now().timestamp_millis();

        if let Some(pos) = boveda.papelera.iter().position(|e| e.id == id) {
            let mut entrada = boveda.papelera.remove(pos);
            entrada.eliminada_en = 0;
            entrada.modificada_en = ahora;
            entrada.revision += 1;
            boveda.entradas.push(entrada);
            self.guardar_a_disco()
        } else if let Some(entrada) = boveda.entradas.iter_mut().find(|e| e.id == id) {
            entrada.eliminada_en = 0;
            entrada.modificada_en = ahora;
            entrada.revision += 1;
            self.guardar_a_disco()
        } else {
            Err(BvdaError::RegistroNoEncontrado(format!("No existe entrada con ID '{id}'")))
        }
    }

    pub fn eliminar_definitivamente(&mut self, id: &str) -> Result<(), BvdaError> {
        let boveda = self.boveda.as_mut().ok_or(BvdaError::AutenticacionFallida)?;
        let len_entradas = boveda.entradas.len();
        let len_papelera = boveda.papelera.len();
        boveda.entradas.retain(|e| e.id != id);
        boveda.papelera.retain(|e| e.id != id);

        if boveda.entradas.len() < len_entradas || boveda.papelera.len() < len_papelera {
            self.guardar_a_disco()
        } else {
            Err(BvdaError::RegistroNoEncontrado(format!("No existe entrada con ID '{id}'")))
        }
    }

    pub fn vaciar_papelera(&mut self) -> Result<usize, BvdaError> {
        let boveda = self.boveda.as_mut().ok_or(BvdaError::AutenticacionFallida)?;
        let inicial_entradas = boveda.entradas.len();
        boveda.entradas.retain(|e| e.eliminada_en == 0);
        let mut eliminadas = inicial_entradas.saturating_sub(boveda.entradas.len());
        eliminadas += boveda.papelera.len();
        boveda.papelera.clear();

        if eliminadas > 0 {
            self.guardar_a_disco()?;
            self.registrar_evento(&format!("Papelera vaciada: {eliminadas} entradas destruidas."));
        }
        Ok(eliminadas)
    }

    pub fn actualizar_ajustes(
        &mut self,
        datos: &SettingsData,
        perfil: PerfilArgon2,
    ) -> Result<(), BvdaError> {
        let boveda = self.boveda.as_mut().ok_or(BvdaError::AutenticacionFallida)?;

        let prefijos: Vec<String> = datos
            .prefijos_subdominios
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let mut puertos = std::collections::HashMap::new();
        for linea in datos.puertos_homelab.lines() {
            let partes: Vec<&str> = linea.splitn(2, ':').collect();
            if partes.len() == 2 {
                let p = partes[0].trim().to_string();
                let n = partes[1].trim().to_string();
                if !p.is_empty() && !n.is_empty() {
                    puertos.insert(p, n);
                }
            }
        }

        if let Some(ref mut c) = boveda.configuracion {
            c.portapapeles_segundos = datos.portapapeles_segundos;
            c.auto_bloqueo_segundos = datos.autobloqueo_segundos;
            c.autodestruccion_intentos_fallidos_max = datos.autodestruccion_intentos;
            c.ocultar_usuario = datos.ocultar_usuario;
            c.ocultar_contrasena = datos.ocultar_contrasena;
            c.ocultar_notas = datos.ocultar_notas;
            c.ocultar_campos = datos.ocultar_campos;
            c.criterio_ordenacion = datos.criterio_ordenacion.to_string();
            c.fijar_anclados_arriba = datos.fijar_anclados_arriba;
            if !prefijos.is_empty() {
                c.prefijos_subdominios = prefijos;
            }
            if !puertos.is_empty() {
                c.puertos_servicios_locales = puertos;
            }
            c.menu_lateral_mostrar_cabecera = datos.menu_lateral_mostrar_cabecera;
            c.menu_lateral_mostrar_pie = datos.menu_lateral_mostrar_pie;
            c.menu_lateral_mostrar_boton_bloquear = datos.menu_lateral_mostrar_boton_bloquear;
            c.menu_lateral_agrupar_items = datos.menu_lateral_agrupar_items;
            c.menu_lateral_sin_bordes = datos.menu_lateral_sin_bordes;
        } else {
            boveda.configuracion = Some(crate::models::vault_config::ConfiguracionBovedaExportable {
                portapapeles_segundos: datos.portapapeles_segundos,
                auto_bloqueo_segundos: datos.autobloqueo_segundos,
                autodestruccion_intentos_fallidos_max: datos.autodestruccion_intentos,
                ocultar_usuario: datos.ocultar_usuario,
                ocultar_contrasena: datos.ocultar_contrasena,
                ocultar_notas: datos.ocultar_notas,
                ocultar_campos: datos.ocultar_campos,
                criterio_ordenacion: datos.criterio_ordenacion.to_string(),
                fijar_anclados_arriba: datos.fijar_anclados_arriba,
                prefijos_subdominios: prefijos,
                puertos_servicios_locales: puertos,
                menu_lateral_mostrar_cabecera: datos.menu_lateral_mostrar_cabecera,
                menu_lateral_mostrar_pie: datos.menu_lateral_mostrar_pie,
                menu_lateral_mostrar_boton_bloquear: datos.menu_lateral_mostrar_boton_bloquear,
                menu_lateral_agrupar_items: datos.menu_lateral_agrupar_items,
                menu_lateral_sin_bordes: datos.menu_lateral_sin_bordes,
                ..Default::default()
            });
        }

        self.perfil_argon2 = perfil;
        self.guardar_a_disco()
    }

    pub fn cambiar_clave_maestra(
        &mut self,
        clave_actual: &str,
        clave_nueva: &str,
        perfil: PerfilArgon2,
    ) -> Result<(), BvdaError> {
        if let Some(ref clave_almacenada) = self.clave_maestra {
            if clave_almacenada.as_str() != clave_actual {
                return Err(BvdaError::AutenticacionFallida);
            }
        } else {
            return Err(BvdaError::AutenticacionFallida);
        }

        self.clave_maestra = Some(Zeroizing::new(clave_nueva.to_string()));
        self.perfil_argon2 = perfil;
        self.guardar_a_disco()
    }

    pub fn obtener_entrada_por_id(&self, id: &str) -> Option<Entrada> {
        self.boveda.as_ref()?.entradas.iter().find(|e| e.id == id).cloned()
    }

    pub fn alternar_expansion_grupo(&mut self, clave: &str) {
        if self.grupos_expandidos.contains(clave) {
            self.grupos_expandidos.remove(clave);
        } else {
            self.grupos_expandidos.insert(clave.to_string());
        }
    }

    pub fn obtener_entradas_filtradas(&self) -> Vec<EntryOverview> {
        let boveda = match &self.boveda {
            Some(b) => b,
            None => return Vec::new(),
        };
        let anclados = boveda
            .configuracion
            .as_ref()
            .map(|c| c.fijar_anclados_arriba)
            .unwrap_or(true);

        let mut entradas = filtrar_entradas(
            boveda,
            &self.filtro_tipo,
            &self.filtro_categoria,
            &self.filtro_perfil,
            &self.filtro_etiqueta,
            &self.termino_busqueda,
            anclados,
        );

        if self.solo_favoritos {
            entradas.retain(|e| e.favorito);
        }

        for e in &mut entradas {
            e.seleccionada_lote = self.seleccion.contiene(e.id.as_str());
        }

        let agrupar = boveda
            .configuracion
            .as_ref()
            .map(|c| c.agrupar_por_sitio)
            .unwrap_or(true);

        aplicar_agrupamiento_servicios(&entradas, agrupar, anclados, &self.grupos_expandidos)
    }

    pub fn obtener_ids_filtrados_reales(&self) -> Vec<String> {
        let boveda = match &self.boveda {
            Some(b) => b,
            None => return Vec::new(),
        };
        let anclados = boveda
            .configuracion
            .as_ref()
            .map(|c| c.fijar_anclados_arriba)
            .unwrap_or(true);
        let mut entradas = filtrar_entradas(
            boveda,
            &self.filtro_tipo,
            &self.filtro_categoria,
            &self.filtro_perfil,
            &self.filtro_etiqueta,
            &self.termino_busqueda,
            anclados,
        );
        if self.solo_favoritos {
            entradas.retain(|e| e.favorito);
        }
        entradas.into_iter().map(|e| e.id.to_string()).collect()
    }

    pub fn obtener_ids_grupo(&self, clave_grupo: &str) -> Vec<String> {
        let boveda = match &self.boveda {
            Some(b) => b,
            None => return Vec::new(),
        };
        let anclados = boveda
            .configuracion
            .as_ref()
            .map(|c| c.fijar_anclados_arriba)
            .unwrap_or(true);
        let mut entradas = filtrar_entradas(
            boveda,
            &self.filtro_tipo,
            &self.filtro_categoria,
            &self.filtro_perfil,
            &self.filtro_etiqueta,
            &self.termino_busqueda,
            anclados,
        );
        if self.solo_favoritos {
            entradas.retain(|e| e.favorito);
        }
        entradas
            .into_iter()
            .filter(|e| !(anclados && e.anclado) && e.servicio.as_str().eq_ignore_ascii_case(clave_grupo))
            .map(|e| e.id.to_string())
            .collect()
    }

    pub fn obtener_resumen_categorias(&self) -> Vec<FilterItem> {
        match &self.boveda {
            Some(b) => calcular_resumen_categorias(b),
            None => Vec::new(),
        }
    }

    pub fn obtener_resumen_perfiles(&self) -> Vec<DropdownOption> {
        match &self.boveda {
            Some(b) => calcular_resumen_perfiles(b),
            None => Vec::new(),
        }
    }

    pub fn obtener_resumen_etiquetas(&self) -> Vec<DropdownOption> {
        match &self.boveda {
            Some(b) => calcular_resumen_etiquetas(b),
            None => Vec::new(),
        }
    }

    pub fn obtener_resumen_tipos(&self) -> Vec<FilterItem> {
        match &self.boveda {
            Some(b) => calcular_resumen_tipos(b),
            None => Vec::new(),
        }
    }

    pub fn obtener_opciones_tipos_dropdown(&self) -> Vec<DropdownOption> {
        match &self.boveda {
            Some(b) => calcular_opciones_tipos_dropdown(b),
            None => Vec::new(),
        }
    }

    pub fn obtener_opciones_categorias_dropdown(&self) -> Vec<DropdownOption> {
        match &self.boveda {
            Some(b) => calcular_opciones_categorias_dropdown(b),
            None => Vec::new(),
        }
    }

    pub fn obtener_detalle_entrada(&self, id: &str) -> Option<EntryDetailData> {
        let boveda = self.boveda.as_ref()?;
        let ocultar_u = boveda.configuracion.as_ref().map(|c| c.ocultar_usuario).unwrap_or(false);
        let ocultar_p = boveda.configuracion.as_ref().map(|c| c.ocultar_contrasena).unwrap_or(true);

        if let Some(e) = boveda.entradas.iter().find(|entry| entry.id == id) {
            return Some(construir_detalle_entrada(e, boveda, ocultar_u, ocultar_p));
        }

        if let Some(clave_grupo) = id.strip_prefix("grp:") {
            let e = boveda
                .entradas
                .iter()
                .filter(|entry| entry.eliminada_en == 0)
                .find(|entry| {
                    let primer_url = entry.urls.first().map(|s| s.as_str()).unwrap_or("");
                    let serv = crate::gui::service_grouper::extraer_nombre_servicio(primer_url, &entry.titulo);
                    serv.eq_ignore_ascii_case(clave_grupo)
                })?;
            return Some(construir_detalle_entrada(e, boveda, ocultar_u, ocultar_p));
        }

        None
    }

    pub fn encontrar_id_por_letra(&self, letra: char) -> Option<String> {
        let filtradas = self.obtener_entradas_filtradas();
        encontrar_id_por_letra(&filtradas, letra)
    }

    pub fn encontrar_indice_por_letra(&self, letra: char) -> Option<usize> {
        let filtradas = self.obtener_entradas_filtradas();
        encontrar_indice_por_letra(&filtradas, letra)
    }
}
