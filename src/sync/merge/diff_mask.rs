/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Máscara booleana inmutable y calculador determinista de diff entre entradas.
//! Representa los 9 segmentos de campos para reflejar con precisión qué datos fueron
//! alterados o incorporados durante una sincronización.
//! Espejo canónico de `MascaraCampos.kt` y `CalculadorMascaraDiff.kt` en Android.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::models::Entrada;

/// Máscara booleana inmutable que representa los 9 segmentos de campos de una entrada
/// para reflejar qué datos fueron modificados o incorporados durante una sincronización.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MascaraCampos {
    /// Cambios en nombre de usuario o identidad de inicio de sesión.
    pub usuario: bool,
    /// Cambios en la contraseña actual o absorción de credenciales en el historial.
    pub clave: bool,
    /// Cambios en semilla TOTP, emisor, algoritmo o códigos 2FA secundarios.
    pub totp: bool,
    /// Cambios en la llave de acceso principal o llaves secundarias.
    pub passkey: bool,
    /// Cambios en direcciones URL de sitios web o dominios asociados.
    pub web: bool,
    /// Cambios en paquetes de aplicaciones Android vinculadas.
    pub app: bool,
    /// Cambios en el texto de notas seguras.
    pub notas: bool,
    /// Cambios en datos nativos de tarjeta, wifi, banco, identidad, etc.
    pub submodelos: bool,
    /// Cambios en campos personalizados o metadatos adicionales.
    pub campos_extra: bool,
}

impl MascaraCampos {
    pub const VACIA: Self = Self {
        usuario: false,
        clave: false,
        totp: false,
        passkey: false,
        web: false,
        app: false,
        notas: false,
        submodelos: false,
        campos_extra: false,
    };

    /// `true` si al menos uno de los 9 segmentos refleja cambios activos.
    pub fn hay_cambios(&self) -> bool {
        self.usuario
            || self.clave
            || self.totp
            || self.passkey
            || self.web
            || self.app
            || self.notas
            || self.submodelos
            || self.campos_extra
    }
}

/// Extrae el paquete de aplicación Android si la URL corresponde a un esquema de app o paquete canónico.
pub fn extraer_paquete_app(url: &str) -> Option<&str> {
    let limpio = url.trim();
    let esquemas = ["androidapp://", "android://", "app://", "package:"];
    for esq in esquemas {
        if limpio.len() >= esq.len() && limpio[..esq.len()].eq_ignore_ascii_case(esq) {
            let resto = &limpio[esq.len()..];
            let fin = resto
                .find(['/', '?'])
                .unwrap_or(resto.len());
            return Some(&resto[..fin]);
        }
    }

    if limpio.contains('.') && !limpio.contains('/') && !limpio.contains(':') {
        let partes: Vec<&str> = limpio.split('.').collect();
        if partes.len() >= 2 {
            let primera = partes[0];
            let mut chars = primera.chars();
            if let Some(primer_char) = chars.next() {
                if (primer_char.is_ascii_alphabetic())
                    && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
                {
                    return Some(limpio);
                }
            }
        }
    }

    None
}

/// Retorna `true` si la URL corresponde a una dirección web estándar y no a un identificador de paquete.
pub fn es_url_web(url: &str) -> bool {
    if extraer_paquete_app(url).is_some() {
        return false;
    }
    let limpio = url.trim();
    limpio.starts_with("http://")
        || limpio.starts_with("https://")
        || limpio.starts_with("HTTP://")
        || limpio.starts_with("HTTPS://")
        || limpio.contains('.')
}

/// Construye una máscara con todos los segmentos que contienen información activa
/// para una entrada clasificada como NUEVA.
pub fn de_entrada_nueva(entrada: &Entrada) -> MascaraCampos {
    let mut tiene_web = false;
    let mut tiene_app = false;

    for u in &entrada.urls {
        if extraer_paquete_app(u).is_some() {
            tiene_app = true;
        } else if es_url_web(u) {
            tiene_web = true;
        }
    }

    let tiene_usuario = !entrada.usuario.trim().is_empty()
        || entrada
            .datos_tarjeta
            .as_ref()
            .is_some_and(|t| !t.titular.trim().is_empty())
        || entrada
            .datos_cuenta_bancaria
            .as_ref()
            .is_some_and(|b| !b.titular.trim().is_empty())
        || entrada
            .datos_identidad
            .as_ref()
            .is_some_and(|i| !i.nombre_completo.trim().is_empty())
        || entrada
            .datos_servidor
            .as_ref()
            .is_some_and(|s| !s.usuario.trim().is_empty());

    let tiene_clave = !entrada.contrasena.trim().is_empty()
        || !entrada.historial_contrasenas.is_empty()
        || entrada
            .datos_wifi
            .as_ref()
            .is_some_and(|w| !w.contrasena.trim().is_empty())
        || entrada.datos_tarjeta.as_ref().is_some_and(|t| {
            !t.cvv.trim().is_empty() || !t.pin.trim().is_empty()
        })
        || entrada
            .datos_servidor
            .as_ref()
            .is_some_and(|s| !s.clave_privada.trim().is_empty())
        || entrada
            .datos_wallet
            .as_ref()
            .is_some_and(|w| !w.clave_privada.trim().is_empty());

    let tiene_totp = entrada
        .secreto_totp
        .as_ref()
        .is_some_and(|s| !s.trim().is_empty())
        || !entrada.totps_secundarios.is_empty();

    let tiene_passkey = entrada.passkey.is_some()
        || !entrada.passkeys_secundarias.is_empty()
        || entrada
            .datos_wallet
            .as_ref()
            .is_some_and(|w| !w.frase_semilla.trim().is_empty())
        || entrada
            .datos_servidor
            .as_ref()
            .is_some_and(|s| !s.clave_publica.trim().is_empty());

    if !tiene_web
        && (entrada
            .datos_servidor
            .as_ref()
            .is_some_and(|s| !s.host.trim().is_empty())
            || entrada
                .datos_wallet
                .as_ref()
                .is_some_and(|w| !w.direccion_publica.trim().is_empty()))
    {
        tiene_web = true;
    }

    let tiene_submodelos = entrada.datos_tarjeta.is_some()
        || entrada.datos_wifi.is_some()
        || entrada.datos_cuenta_bancaria.is_some()
        || entrada.datos_identidad.is_some()
        || entrada.datos_servidor.is_some()
        || entrada.datos_wallet.is_some()
        || entrada.datos_contacto.is_some();

    MascaraCampos {
        usuario: tiene_usuario,
        clave: tiene_clave,
        totp: tiene_totp,
        passkey: tiene_passkey,
        web: tiene_web,
        app: tiene_app,
        notas: !entrada.notas.trim().is_empty(),
        submodelos: tiene_submodelos,
        campos_extra: !entrada.campos_personalizados.is_empty(),
    }
}

/// Compara campo a campo dos versiones de una misma entrada y devuelve la máscara
/// con los segmentos exactamente modificados.
pub fn calcular_diff(previa: &Entrada, nueva: &Entrada) -> MascaraCampos {
    let clave = (previa.contrasena != nueva.contrasena)
        || (previa.historial_contrasenas != nueva.historial_contrasenas);

    let totp = (previa.secreto_totp != nueva.secreto_totp)
        || (previa.totp_emisor != nueva.totp_emisor)
        || (previa.totp_digitos != nueva.totp_digitos)
        || (previa.totp_periodo != nueva.totp_periodo)
        || (previa.totp_algoritmo != nueva.totp_algoritmo)
        || (previa.totps_secundarios != nueva.totps_secundarios);

    let passkey = (previa.passkey != nueva.passkey)
        || (previa.passkeys_secundarias != nueva.passkeys_secundarias);

    let webs_previas: HashSet<&str> = previa
        .urls
        .iter()
        .map(|s| s.as_str())
        .filter(|u| es_url_web(u))
        .collect();
    let webs_nuevas: HashSet<&str> = nueva
        .urls
        .iter()
        .map(|s| s.as_str())
        .filter(|u| es_url_web(u))
        .collect();
    let web = webs_previas != webs_nuevas;

    let apps_previas: HashSet<&str> = previa
        .urls
        .iter()
        .map(|s| s.as_str())
        .filter_map(extraer_paquete_app)
        .collect();
    let apps_nuevas: HashSet<&str> = nueva
        .urls
        .iter()
        .map(|s| s.as_str())
        .filter_map(extraer_paquete_app)
        .collect();
    let app = apps_previas != apps_nuevas;

    let submodelos = (previa.datos_tarjeta != nueva.datos_tarjeta)
        || (previa.datos_wifi != nueva.datos_wifi)
        || (previa.datos_cuenta_bancaria != nueva.datos_cuenta_bancaria)
        || (previa.datos_identidad != nueva.datos_identidad)
        || (previa.datos_servidor != nueva.datos_servidor)
        || (previa.datos_wallet != nueva.datos_wallet)
        || (previa.datos_contacto != nueva.datos_contacto);

    MascaraCampos {
        usuario: previa.usuario != nueva.usuario,
        clave,
        totp,
        passkey,
        web,
        app,
        notas: previa.notas != nueva.notas,
        submodelos,
        campos_extra: previa.campos_personalizados != nueva.campos_personalizados,
    }
}
