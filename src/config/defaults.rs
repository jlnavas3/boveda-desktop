/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Fuente única de la verdad para TODOS los valores predeterminados del sistema (Zero-Hardcoding).
//! Replicado directamente de `AjustesDefaults.kt` de Bóveda Local Android.

use crate::gui::SettingsData;

pub mod seguridad {
    pub const AUTO_BLOQUEO_SEGUNDOS: i32 = 60;
    pub const PORTAPAPELES_SEGUNDOS: i32 = 30;
    pub const PERFIL_ARGON2: &str = "estandar";
    pub const AUTODESTRUCCION_INTENTOS_FALLIDOS_MAX: i32 = 0;
    pub const FRENO_INTENTOS_GRATIS: i32 = 5;
    pub const FRENO_SEGUNDOS_BASE: i64 = 5;
    pub const FRENO_SEGUNDOS_MAX: i64 = 300;
}

pub mod apariencia {
    pub const TEMA_APP: &str = "sistema";
    pub const MODO_OSCURO: &str = "oscuro";
    pub const COLOR_ACENTO: &str = "menta";
    pub const CURVATURA_ESQUINAS_PX: f32 = 6.0;
    pub const ESTILO_INDICADORES: &str = "iconos";
    pub const TAMANIO_INDICADORES_PX: f32 = 12.0;
    pub const OPACIDAD_INDICADORES: f32 = 1.0;
    pub const NOMBRE_BOVEDA_DEFECTO: &str = "Mi Bóveda Principal";
}

pub mod lista_formatos {
    pub const DENSIDAD_LISTA: &str = "predeterminada";
    pub const CRITERIO_ORDENACION: &str = "NOMBRE_AZ";
    pub const FIJAR_ANCLADOS_ARRIBA: bool = true;
    pub const AGRUPAR_POR_SITIO: bool = false;
    pub const OCULTAR_USUARIO: bool = false;
    pub const OCULTAR_CONTRASENA: bool = true;
    pub const OCULTAR_NOTAS: bool = false;
    pub const OCULTAR_CAMPOS: bool = true;
}

pub mod niagara {
    pub const MOSTRAR: bool = true;
    pub const EFECTO_OLA: bool = true;
    pub const AMPLITUD_OLA_PX: f32 = 24.0;
    pub const RADIO_OLA_PX: f32 = 110.0;
    pub const ESCALA_LETRAS: f32 = 1.6;
    pub const MOSTRAR_CIRCULO: bool = true;
    pub const DIAMETRO_CIRCULO_PX: f32 = 48.0;
    pub const DESPLAZAMIENTO_CIRCULO_PX: f32 = -56.0;
    pub const ANCHO_TACTIL_PX: f32 = 26.0;
    pub const OPACIDAD_INACTIVAS: f32 = 0.6;
    pub const INCLUIR_ENIE: bool = true;
}

pub mod escalado_ventana {
    /// Resolución base de referencia de diseño (Full HD 1920x1080 = factor 1.0).
    pub const ANCHO_REFERENCIA_FHD: u32 = 1920;
    pub const ALTO_REFERENCIA_FHD: u32 = 1080;

    /// Umbrales de resolución horizontal para auto-adaptación.
    pub const UMBRAL_ANCHO_HD: u32 = 1366;
    pub const UMBRAL_ANCHO_QHD: u32 = 2500;
    pub const UMBRAL_ANCHO_UHD_4K: u32 = 3400;

    /// Factores de escala base automáticos según la resolución física.
    pub const ESCALA_AUTO_HD_720P: f32 = 0.75;
    pub const ESCALA_AUTO_FHD_1080P: f32 = 1.00;
    pub const ESCALA_AUTO_QHD_1440P: f32 = 1.25;
    pub const ESCALA_AUTO_UHD_2160P: f32 = 1.75;

    /// Parámetros del multiplicador de zoom manual del usuario (Ctrl +, Ctrl -, Ctrl 0).
    pub const MULTIPLICADOR_USUARIO_DEFECTO: f32 = 1.00;
    pub const MULTIPLICADOR_USUARIO_MIN: f32 = 0.60;
    pub const MULTIPLICADOR_USUARIO_MAX: f32 = 2.00;
    pub const PASO_ZOOM_USUARIO: f32 = 0.10;

    /// Límites absolutos de seguridad del factor de escala combinado enviado a Slint.
    pub const FACTOR_FINAL_MIN: f32 = 0.50;
    pub const FACTOR_FINAL_MAX: f32 = 3.00;
}

pub mod normalizacion_titulos {
    pub const PREFIJOS_SUBDOMINIOS: &[&str] = &[
        "account", "accounts", "access", "login", "signin", "auth", "sso",
        "myaccount", "identity", "id", "passport", "portal", "webmail",
        "clientes", "servicios", "online", "banca", "app", "mobile", "m", "www",
        "api", "api-dashboard", "search", "my", "secure", "connect", "admin",
        "dashboard", "cloud", "console", "member", "members", "user", "users",
        "dev", "stage", "staging", "test", "beta", "oauth",
    ];

    pub const TLDS_DESCARTABLES: &[&str] = &[
        "com", "net", "org", "io", "co", "ec", "es", "pe", "cl", "ar", "mx",
        "app", "dev", "ai", "tech", "me", "info", "online", "tv", "it",
        "edu", "gob", "guru", "cloud", "art", "store", "shop", "xyz", "site", "design",
    ];

    pub const PUERTOS_PREDETERMINADOS: &[(&str, &str)] = &[
        ("8006", "Proxmox"),
        ("8080", "Servidor Web"),
        ("8096", "Jellyfin"),
        ("8123", "Home Assistant"),
        ("9000", "Portainer"),
        ("9443", "Portainer SSL"),
        ("5000", "Synology DSM"),
        ("5001", "Synology DSM SSL"),
        ("32400", "Plex"),
    ];

    pub fn puertos_servicios_locales() -> Vec<(String, String)> {
        PUERTOS_PREDETERMINADOS
            .iter()
            .map(|(p, s)| (p.to_string(), s.to_string()))
            .collect()
    }
}

pub mod menu_lateral {
    pub const MOSTRAR_CABECERA: bool = true;
    pub const MOSTRAR_PIE: bool = true;
    pub const MOSTRAR_BOTON_BLOQUEAR: bool = true;
    pub const AGRUPAR_ITEMS: bool = false;
    pub const SIN_BORDES: bool = false;
}

/// Genera los valores predeterminados canónicos en la estructura `SettingsData` de Slint.
pub fn valores_predeterminados() -> SettingsData {
    let prefijos = normalizacion_titulos::PREFIJOS_SUBDOMINIOS.join(", ");
    let mut puertos_str = String::new();
    for (p, s) in normalizacion_titulos::PUERTOS_PREDETERMINADOS {
        puertos_str.push_str(&format!("{p}: {s}\n"));
    }

    SettingsData {
        portapapeles_segundos: seguridad::PORTAPAPELES_SEGUNDOS,
        autobloqueo_segundos: seguridad::AUTO_BLOQUEO_SEGUNDOS,
        autodestruccion_intentos: seguridad::AUTODESTRUCCION_INTENTOS_FALLIDOS_MAX,
        perfil_argon2: "ESTANDAR".into(),
        biometria_activa: false,
        modo_tema: apariencia::MODO_OSCURO.into(),
        color_acento: apariencia::COLOR_ACENTO.into(),
        nombre_boveda: apariencia::NOMBRE_BOVEDA_DEFECTO.into(),
        ocultar_usuario: lista_formatos::OCULTAR_USUARIO,
        ocultar_contrasena: lista_formatos::OCULTAR_CONTRASENA,
        ocultar_notas: lista_formatos::OCULTAR_NOTAS,
        ocultar_campos: lista_formatos::OCULTAR_CAMPOS,
        criterio_ordenacion: lista_formatos::CRITERIO_ORDENACION.into(),
        fijar_anclados_arriba: lista_formatos::FIJAR_ANCLADOS_ARRIBA,
        mostrar_ola_niagara: niagara::MOSTRAR,
        incluir_enie_niagara: niagara::INCLUIR_ENIE,
        prefijos_subdominios: prefijos.into(),
        puertos_homelab: puertos_str.trim().into(),
        menu_lateral_mostrar_cabecera: menu_lateral::MOSTRAR_CABECERA,
        menu_lateral_mostrar_pie: menu_lateral::MOSTRAR_PIE,
        menu_lateral_mostrar_boton_bloquear: menu_lateral::MOSTRAR_BOTON_BLOQUEAR,
        menu_lateral_agrupar_items: menu_lateral::AGRUPAR_ITEMS,
        menu_lateral_sin_bordes: menu_lateral::SIN_BORDES,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valores_predeterminados_canónicos() {
        let defaults = valores_predeterminados();
        assert_eq!(defaults.autobloqueo_segundos, 60);
        assert_eq!(defaults.portapapeles_segundos, 30);
        assert_eq!(defaults.autodestruccion_intentos, 0);
        assert_eq!(defaults.color_acento.as_str(), "menta");
        assert_eq!(defaults.criterio_ordenacion.as_str(), "NOMBRE_AZ");
        assert!(defaults.fijar_anclados_arriba);
        assert!(defaults.incluir_enie_niagara);
    }
}
