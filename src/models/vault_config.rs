/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Configuración portable de la bóveda (ConfiguracionBovedaExportable).
//! Serializada dentro del archivo .bvda para conservar parámetros de seguridad,
//! normalización y reglas funcionales entre dispositivos (Zero-Hardcoding).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Modelo de datos que encapsula todas las configuraciones funcionales,
/// reglas de URLs, homelab, puertos, generador y seguridad incluidas en el archivo `.bvda`.
/// Idéntico a `ConfiguracionBovedaExportable.kt` y calibrado con `AjustesDefaults.kt`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfiguracionBovedaExportable {
    // 1. Reglas de URLs, Redes y Normalización de Títulos
    #[serde(default = "default_prefijos_subdominios")]
    pub prefijos_subdominios: Vec<String>,
    #[serde(default = "default_tlds_descartables")]
    pub tlds_descartables: Vec<String>,
    #[serde(default)]
    pub marcas_personalizadas: HashMap<String, String>,
    #[serde(default = "default_puertos_servicios_locales")]
    pub puertos_servicios_locales: HashMap<String, String>,
    #[serde(default = "default_octetos_router")]
    pub octetos_router: Vec<i32>,
    #[serde(default = "default_plantilla_router_ip")]
    pub plantilla_router_ip: String,
    #[serde(default = "default_plantilla_servidor_ip")]
    pub plantilla_servidor_ip: String,
    #[serde(default = "default_formato_colision_titulos")]
    pub formato_colision_titulos: String,
    #[serde(default = "default_true")]
    pub respetar_titulos_personalizados: bool,
    #[serde(default = "default_rutas_servicios_locales")]
    pub rutas_servicios_locales: HashMap<String, String>,

    // 2. Seguridad Funcional
    #[serde(default = "default_auto_bloqueo_segundos")]
    pub auto_bloqueo_segundos: i32,
    #[serde(default = "default_portapapeles_segundos")]
    pub portapapeles_segundos: i32,
    #[serde(default = "default_perfil_argon2")]
    pub perfil_argon2: String,
    #[serde(default = "default_freno_intentos_gratis")]
    pub freno_intentos_gratis: i32,
    #[serde(default = "default_freno_segundos_base")]
    pub freno_segundos_base: i64,
    #[serde(default = "default_freno_segundos_max")]
    pub freno_segundos_max: i64,
    #[serde(default)]
    pub autodestruccion_intentos_fallidos_max: i32,

    // 3. Privacidad y Seguridad Visual
    #[serde(default)]
    pub seguridad_visual_activa: bool,
    #[serde(default = "default_estilo_ocultamiento")]
    pub estilo_ocultamiento_visual: String,
    #[serde(default = "default_tiempo_auto_ocultar")]
    pub tiempo_auto_ocultar_segundos: i32,
    #[serde(default = "default_true")]
    pub ocultar_usuario: bool,
    #[serde(default = "default_true")]
    pub ocultar_contrasena: bool,
    #[serde(default)]
    pub ocultar_totp: bool,
    #[serde(default = "default_true")]
    pub ocultar_notas: bool,
    #[serde(default = "default_true")]
    pub ocultar_campos: bool,

    // 4. Lista y Formatos
    #[serde(default = "default_densidad_lista")]
    pub densidad_lista: String,
    #[serde(default = "default_criterio_ordenacion")]
    pub criterio_ordenacion: String,
    #[serde(default = "default_true")]
    pub agrupar_por_sitio: bool,
    #[serde(default = "default_true")]
    pub fijar_anclados_arriba: bool,
    #[serde(default = "default_modo_identidades")]
    pub modo_identidades: String,
    #[serde(default = "default_jerarquia_organizacion")]
    pub jerarquia_organizacion: String,
    #[serde(default = "default_formato_fecha")]
    pub formato_fecha: String,
    #[serde(default = "default_formato_hora")]
    pub formato_hora: String,
    #[serde(default = "default_formato_telefono")]
    pub formato_telefono: String,
    #[serde(default = "default_separador_decimal")]
    pub separador_decimal: String,

    // 5. Generador de Contraseñas y 2FA
    #[serde(default = "default_totp_manual_digitos")]
    pub totp_manual_digitos: i32,
    #[serde(default = "default_totp_manual_periodo")]
    pub totp_manual_periodo: i32,
    #[serde(default = "default_totp_manual_algoritmo")]
    pub totp_manual_algoritmo: String,
    #[serde(default = "default_generador_longitud")]
    pub generador_longitud: i32,
    #[serde(default = "default_true")]
    pub generador_mayusculas: bool,
    #[serde(default = "default_true")]
    pub generador_minusculas: bool,
    #[serde(default = "default_true")]
    pub generador_digitos: bool,
    #[serde(default = "default_true")]
    pub generador_simbolos: bool,
    #[serde(default = "default_true")]
    pub generador_excluir_ambiguos: bool,
    #[serde(default = "default_alias_delimitador")]
    pub alias_correo_delimitador: String,
    #[serde(default = "default_alias_longitud_sufijo")]
    pub alias_correo_longitud_sufijo: i32,
    #[serde(default)]
    pub alias_correo_base_predeterminado: String,

    // 6. Ciclo de Vida y Retención
    #[serde(default = "default_dias_retencion_papelera")]
    pub dias_retencion_papelera: i32,
    #[serde(default = "default_max_historial_contrasenas")]
    pub max_historial_contrasenas_por_entrada: i32,
    #[serde(default = "default_recordatorio_exportacion_dias")]
    pub recordatorio_exportacion_dias: i32,

    // 7. Formas y Geometría (AjustesDefaults.Formas)
    #[serde(default = "default_curvatura_esquinas_dp")]
    pub curvatura_esquinas_dp: f32,
    #[serde(default = "default_grosor_borde_dp")]
    pub grosor_borde_dp: f32,
    #[serde(default = "default_espaciado_componentes_dp")]
    pub espaciado_componentes_dp: f32,
    #[serde(default = "default_estilo_borde")]
    pub estilo_borde: String,

    // 8. Tipografía (AjustesDefaults.Tipografia)
    #[serde(default = "default_escala_texto")]
    pub escala_texto: f32,
    #[serde(default = "default_familia_fuente")]
    pub familia_fuente: String,
    #[serde(default = "default_peso_texto")]
    pub peso_texto: String,

    // 9. Personalización del Menú Lateral (AjustesDefaults.MenuLateral)
    #[serde(default = "default_true")]
    pub menu_lateral_mostrar_cabecera: bool,
    #[serde(default = "default_true")]
    pub menu_lateral_mostrar_pie: bool,
    #[serde(default = "default_true")]
    pub menu_lateral_mostrar_boton_bloquear: bool,
    #[serde(default)]
    pub menu_lateral_agrupar_items: bool,
    #[serde(default)]
    pub menu_lateral_sin_bordes: bool,

    // 10. Avisos y Asistencia (AjustesDefaults.Avisos)
    #[serde(default = "default_true")]
    pub mostrar_campana_cabecera: bool,
    #[serde(default = "default_true")]
    pub avisar_duplicados: bool,
    #[serde(default = "default_true")]
    pub avisar_titulos_sin_normalizar: bool,
    #[serde(default = "default_true")]
    pub avisar_salud_contrasenas: bool,
    #[serde(default = "default_true")]
    pub avisar_respaldo_pendiente: bool,

    // 11. Formatos Específicos (AjustesDefaults.ListaFormatos)
    #[serde(default = "default_formato_tarjeta")]
    pub formato_tarjeta: String,
    #[serde(default = "default_formato_expiracion_tarjeta")]
    pub formato_expiracion_tarjeta: String,
    #[serde(default = "default_formato_iban")]
    pub formato_iban: String,
}

fn default_true() -> bool {
    true
}

fn default_prefijos_subdominios() -> Vec<String> {
    vec![
        "account".into(), "accounts".into(), "access".into(), "login".into(),
        "signin".into(), "auth".into(), "sso".into(), "myaccount".into(),
        "identity".into(), "id".into(), "passport".into(), "portal".into(),
        "webmail".into(), "clientes".into(), "servicios".into(), "online".into(),
        "banca".into(), "app".into(), "mobile".into(), "m".into(), "www".into(),
        "api".into(), "api-dashboard".into(), "search".into(), "my".into(),
        "secure".into(), "connect".into(), "admin".into(), "dashboard".into(),
        "cloud".into(), "console".into(), "member".into(), "members".into(),
        "user".into(), "users".into(), "dev".into(), "stage".into(),
        "staging".into(), "test".into(), "beta".into(), "oauth".into(),
    ]
}

fn default_tlds_descartables() -> Vec<String> {
    vec![
        "com".into(), "net".into(), "org".into(), "io".into(), "co".into(),
        "ec".into(), "es".into(), "pe".into(), "cl".into(), "ar".into(),
        "mx".into(), "app".into(), "dev".into(), "ai".into(), "tech".into(),
        "me".into(), "info".into(), "online".into(), "tv".into(), "it".into(),
        "edu".into(), "gob".into(), "guru".into(), "cloud".into(), "art".into(),
        "store".into(), "shop".into(), "xyz".into(), "site".into(), "design".into(),
    ]
}

fn default_puertos_servicios_locales() -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("8006".into(), "Proxmox".into());
    map.insert("9000".into(), "Portainer".into());
    map.insert("9443".into(), "Portainer".into());
    map.insert("8123".into(), "Home Assistant".into());
    map.insert("32400".into(), "Plex".into());
    map.insert("8096".into(), "Jellyfin".into());
    map.insert("5000".into(), "Synology".into());
    map.insert("5001".into(), "Synology".into());
    map.insert("8080".into(), "Servidor Web".into());
    map
}

fn default_rutas_servicios_locales() -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("phpmyadmin".into(), "phpMyAdmin".into());
    map.insert("portal".into(), "Portal Web".into());
    map
}

fn default_octetos_router() -> Vec<i32> {
    vec![1, 254]
}

fn default_plantilla_router_ip() -> String {
    "Router ({ip})".into()
}

fn default_plantilla_servidor_ip() -> String {
    "Servidor ({ip})".into()
}

fn default_formato_colision_titulos() -> String {
    "MINIMALISTA".into()
}

fn default_auto_bloqueo_segundos() -> i32 {
    60
}

fn default_portapapeles_segundos() -> i32 {
    30
}

fn default_perfil_argon2() -> String {
    "estandar".into()
}

fn default_freno_intentos_gratis() -> i32 {
    5
}

fn default_freno_segundos_base() -> i64 {
    5
}

fn default_freno_segundos_max() -> i64 {
    300
}

fn default_estilo_ocultamiento() -> String {
    "desenfoque".into()
}

fn default_tiempo_auto_ocultar() -> i32 {
    10
}

fn default_densidad_lista() -> String {
    "predeterminada".into()
}

fn default_criterio_ordenacion() -> String {
    "NOMBRE_AZ".into()
}

fn default_modo_identidades() -> String {
    "chips".into()
}

fn default_jerarquia_organizacion() -> String {
    "identidad_sobre_categoria".into()
}

fn default_formato_fecha() -> String {
    "DD/MM/AAAA".into()
}

fn default_formato_hora() -> String {
    "24h".into()
}

fn default_formato_telefono() -> String {
    "### ### ####".into()
}

fn default_separador_decimal() -> String {
    ".".into()
}

fn default_totp_manual_digitos() -> i32 {
    6
}

fn default_totp_manual_periodo() -> i32 {
    30
}

fn default_totp_manual_algoritmo() -> String {
    "HmacSHA1".into()
}

fn default_generador_longitud() -> i32 {
    20
}

fn default_alias_delimitador() -> String {
    "+".into()
}

fn default_alias_longitud_sufijo() -> i32 {
    6
}

fn default_dias_retencion_papelera() -> i32 {
    30
}

fn default_max_historial_contrasenas() -> i32 {
    5
}

fn default_recordatorio_exportacion_dias() -> i32 {
    30
}

fn default_curvatura_esquinas_dp() -> f32 {
    6.0
}

fn default_grosor_borde_dp() -> f32 {
    0.8
}

fn default_espaciado_componentes_dp() -> f32 {
    6.0
}

fn default_estilo_borde() -> String {
    "sutil".into()
}

fn default_escala_texto() -> f32 {
    1.0
}

fn default_familia_fuente() -> String {
    "sans".into()
}

fn default_peso_texto() -> String {
    "normal".into()
}

fn default_formato_tarjeta() -> String {
    "4x4".into()
}

fn default_formato_expiracion_tarjeta() -> String {
    "MM/AA".into()
}

fn default_formato_iban() -> String {
    "bloques_4".into()
}

impl Default for ConfiguracionBovedaExportable {
    fn default() -> Self {
        Self {
            prefijos_subdominios: default_prefijos_subdominios(),
            tlds_descartables: default_tlds_descartables(),
            marcas_personalizadas: HashMap::new(),
            puertos_servicios_locales: default_puertos_servicios_locales(),
            octetos_router: default_octetos_router(),
            plantilla_router_ip: default_plantilla_router_ip(),
            plantilla_servidor_ip: default_plantilla_servidor_ip(),
            formato_colision_titulos: default_formato_colision_titulos(),
            respetar_titulos_personalizados: true,
            rutas_servicios_locales: default_rutas_servicios_locales(),
            auto_bloqueo_segundos: default_auto_bloqueo_segundos(),
            portapapeles_segundos: default_portapapeles_segundos(),
            perfil_argon2: default_perfil_argon2(),
            freno_intentos_gratis: default_freno_intentos_gratis(),
            freno_segundos_base: default_freno_segundos_base(),
            freno_segundos_max: default_freno_segundos_max(),
            autodestruccion_intentos_fallidos_max: 0,
            seguridad_visual_activa: false,
            estilo_ocultamiento_visual: default_estilo_ocultamiento(),
            tiempo_auto_ocultar_segundos: default_tiempo_auto_ocultar(),
            ocultar_usuario: true,
            ocultar_contrasena: true,
            ocultar_totp: false,
            ocultar_notas: true,
            ocultar_campos: true,
            densidad_lista: default_densidad_lista(),
            criterio_ordenacion: default_criterio_ordenacion(),
            agrupar_por_sitio: true,
            fijar_anclados_arriba: true,
            modo_identidades: default_modo_identidades(),
            jerarquia_organizacion: default_jerarquia_organizacion(),
            formato_fecha: default_formato_fecha(),
            formato_hora: default_formato_hora(),
            formato_telefono: default_formato_telefono(),
            separador_decimal: default_separador_decimal(),
            totp_manual_digitos: default_totp_manual_digitos(),
            totp_manual_periodo: default_totp_manual_periodo(),
            totp_manual_algoritmo: default_totp_manual_algoritmo(),
            generador_longitud: default_generador_longitud(),
            generador_mayusculas: true,
            generador_minusculas: true,
            generador_digitos: true,
            generador_simbolos: true,
            generador_excluir_ambiguos: true,
            alias_correo_delimitador: default_alias_delimitador(),
            alias_correo_longitud_sufijo: default_alias_longitud_sufijo(),
            alias_correo_base_predeterminado: String::new(),
            dias_retencion_papelera: default_dias_retencion_papelera(),
            max_historial_contrasenas_por_entrada: default_max_historial_contrasenas(),
            recordatorio_exportacion_dias: default_recordatorio_exportacion_dias(),
            curvatura_esquinas_dp: default_curvatura_esquinas_dp(),
            grosor_borde_dp: default_grosor_borde_dp(),
            espaciado_componentes_dp: default_espaciado_componentes_dp(),
            estilo_borde: default_estilo_borde(),
            escala_texto: default_escala_texto(),
            familia_fuente: default_familia_fuente(),
            peso_texto: default_peso_texto(),
            menu_lateral_mostrar_cabecera: true,
            menu_lateral_mostrar_pie: true,
            menu_lateral_mostrar_boton_bloquear: true,
            menu_lateral_agrupar_items: false,
            menu_lateral_sin_bordes: false,
            mostrar_campana_cabecera: true,
            avisar_duplicados: true,
            avisar_titulos_sin_normalizar: true,
            avisar_salud_contrasenas: true,
            avisar_respaldo_pendiente: true,
            formato_tarjeta: default_formato_tarjeta(),
            formato_expiracion_tarjeta: default_formato_expiracion_tarjeta(),
            formato_iban: default_formato_iban(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults_fieles_a_android() {
        let config = ConfiguracionBovedaExportable::default();
        assert_eq!(config.auto_bloqueo_segundos, 60);
        assert_eq!(config.portapapeles_segundos, 30);
        assert_eq!(config.perfil_argon2, "estandar");
        assert_eq!(config.criterio_ordenacion, "NOMBRE_AZ");
        assert!(config.prefijos_subdominios.contains(&"login".to_string()));
        assert_eq!(config.puertos_servicios_locales.get("8006").map(|s| s.as_str()), Some("Proxmox"));
    }

    #[test]
    fn test_serializacion_camel_case() {
        let config = ConfiguracionBovedaExportable::default();
        let json = serde_json::to_string(&config).expect("debe serializar");
        assert!(json.contains("autoBloqueoSegundos"));
        assert!(json.contains("portapapelesSegundos"));
        assert!(json.contains("criterioOrdenacion"));
    }
}
