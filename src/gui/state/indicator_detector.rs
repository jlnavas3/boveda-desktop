/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Detección de indicadores y segmentos de presencia de datos en tarjetas.
//! Réplica matemática y semántica de `SegmentoIndicadorDato.kt` de Bóveda Local Android.

use crate::models::entrada::Entrada;
use crate::models::tipo_entrada::TipoEntrada;

/// Estructura atómica con las banderas de presencia de contenido en una entrada.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IndicadoresEntrada {
    pub tiene_usuario: bool,
    pub tiene_password: bool,
    pub tiene_totp: bool,
    pub tiene_passkey: bool,
    pub tiene_url: bool,
    pub tiene_app: bool,
    pub tiene_notas: bool,
}

/// Extrae si un enlace corresponde a un paquete de aplicación Android.
/// Idéntico a `LanzadorEnlaces.extraerPaquete` de Android.
pub fn es_paquete_app(url: &str) -> bool {
    let limpio = url.trim();
    if limpio.is_empty() {
        return false;
    }

    let es_esquema_android = limpio.starts_with("android://")
        || limpio.starts_with("https://android//")
        || limpio.starts_with("http://android//")
        || limpio.starts_with("android-app://")
        || limpio.starts_with("androidapp://")
        || limpio.starts_with("app://")
        || limpio.starts_with("package:");

    if es_esquema_android {
        return true;
    }

    // Comprobación de formato de paquete Java/Android directo (e.g., com.amazon.mShop.android.shopping)
    if !limpio.contains('/') && !limpio.contains(':') && !limpio.contains('?') && !limpio.contains(' ') && !limpio.contains('@') {
        let partes: Vec<&str> = limpio.split('.').collect();
        if partes.len() >= 2 {
            let primera = partes[0].to_lowercase();
            let ultima = partes[partes.len() - 1].to_lowercase();

            // Un paquete nunca empieza con www
            if primera == "www" {
                return false;
            }

            const PREFIJOS_PAQUETE: &[&str] = &[
                "com", "org", "net", "edu", "gov", "mil", "android", "google", "io", "es", "de",
                "fr", "uk", "jp", "br", "ru", "in", "app", "me", "tech", "dev", "pl", "us", "ch",
                "nl", "se", "no", "fi",
            ];

            const TLDS_WEB: &[&str] = &[
                "com", "org", "net", "edu", "gov", "mil", "int", "eu", "ar", "es", "mx", "co",
                "pe", "cl", "ec", "ve", "uy", "br", "uk", "de", "fr", "it", "ca", "us", "cn",
                "jp", "ru", "in", "info", "biz", "pro", "xyz", "online", "site", "website",
                "tech", "store", "shop", "blog", "app", "dev", "io", "ai", "tv", "me", "local",
            ];

            let es_tld_comun = TLDS_WEB.contains(&ultima.as_str());

            if partes.len() == 2 && es_tld_comun && !TLDS_WEB.contains(&primera.as_str()) {
                return false;
            }

            if partes.len() == 2 {
                return PREFIJOS_PAQUETE.contains(&primera.as_str()) && !es_tld_comun;
            }

            // Para 3 o más partes:
            if es_tld_comun && !PREFIJOS_PAQUETE.contains(&primera.as_str()) {
                return false;
            }

            return true;
        }
    }

    false
}

/// Evalúa una entrada y devuelve las banderas de sus 6 segmentos canónicos de indicadores más notas.
/// Replica la lógica de `recordarSegmentosEntrada(entrada: Entrada)` de Android.
pub fn detectar_indicadores(entrada: &Entrada) -> IndicadoresEntrada {
    let mut tiene_web = false;
    let mut tiene_app = false;

    for u in &entrada.urls {
        if tiene_web && tiene_app {
            break;
        }
        if es_paquete_app(u) {
            tiene_app = true;
        } else if u.contains('.') || u.to_lowercase().starts_with("http") {
            tiene_web = true;
        }
    }

    let tiene_notas = !entrada.notas.trim().is_empty();

    match entrada.tipo {
        TipoEntrada::Wifi => {
            let wifi = entrada.datos_wifi.as_ref();
            let tiene_contrasena = wifi.map(|w| !w.contrasena.trim().is_empty()).unwrap_or(false);
            let tiene_seguridad = wifi
                .map(|w| {
                    !w.tipo_seguridad.trim().is_empty()
                        && !w.tipo_seguridad.eq_ignore_ascii_case("abierta")
                })
                .unwrap_or(false);
            let tiene_ssid = wifi.map(|w| !w.ssid.trim().is_empty()).unwrap_or(false);

            IndicadoresEntrada {
                tiene_usuario: false,
                tiene_password: tiene_contrasena,
                tiene_totp: tiene_seguridad,
                tiene_passkey: false,
                tiene_url: tiene_ssid,
                tiene_app,
                tiene_notas,
            }
        }
        TipoEntrada::Tarjeta => {
            let tarjeta = entrada.datos_tarjeta.as_ref();
            let tiene_titular = tarjeta.map(|t| !t.titular.trim().is_empty()).unwrap_or(false);
            let tiene_secretos = tarjeta
                .map(|t| !t.cvv.trim().is_empty() || !t.pin.trim().is_empty())
                .unwrap_or(false);
            let tiene_vencimiento = tarjeta.map(|t| !t.vencimiento.trim().is_empty()).unwrap_or(false);
            let tiene_banco = tarjeta
                .map(|t| !t.marca.trim().is_empty() || !t.tipo.trim().is_empty())
                .unwrap_or(false)
                || tiene_web;

            IndicadoresEntrada {
                tiene_usuario: tiene_titular,
                tiene_password: tiene_secretos,
                tiene_totp: tiene_vencimiento,
                tiene_passkey: false,
                tiene_url: tiene_banco,
                tiene_app,
                tiene_notas,
            }
        }
        TipoEntrada::CuentaBancaria => {
            let cuenta = entrada.datos_cuenta_bancaria.as_ref();
            let tiene_titular = cuenta.map(|c| !c.titular.trim().is_empty()).unwrap_or(false);
            let tiene_banco = cuenta.map(|c| !c.banco.trim().is_empty()).unwrap_or(false) || tiene_web;

            IndicadoresEntrada {
                tiene_usuario: tiene_titular,
                tiene_password: false,
                tiene_totp: false,
                tiene_passkey: false,
                tiene_url: tiene_banco,
                tiene_app,
                tiene_notas,
            }
        }
        TipoEntrada::Identidad => {
            let id = entrada.datos_identidad.as_ref();
            let tiene_nombre = id.map(|i| !i.nombre_completo.trim().is_empty()).unwrap_or(false);
            let tiene_vencimiento = id.map(|i| !i.caducidad.trim().is_empty()).unwrap_or(false);
            let tiene_pais = id.map(|i| !i.pais_emisor.trim().is_empty()).unwrap_or(false) || tiene_web;

            IndicadoresEntrada {
                tiene_usuario: tiene_nombre,
                tiene_password: false,
                tiene_totp: tiene_vencimiento,
                tiene_passkey: false,
                tiene_url: tiene_pais,
                tiene_app,
                tiene_notas,
            }
        }
        TipoEntrada::Servidor => {
            let serv = entrada.datos_servidor.as_ref();
            let tiene_usuario = serv.map(|s| !s.usuario.trim().is_empty()).unwrap_or(false)
                || !entrada.usuario.trim().is_empty();
            let tiene_clave_privada = serv.map(|s| !s.clave_privada.trim().is_empty()).unwrap_or(false)
                || !entrada.contrasena.is_empty();
            let tiene_2fa = entrada.tiene_totp();
            let tiene_clave_publica = serv.map(|s| !s.clave_publica.trim().is_empty()).unwrap_or(false);
            let tiene_host = serv.map(|s| !s.host.trim().is_empty()).unwrap_or(false) || tiene_web;

            IndicadoresEntrada {
                tiene_usuario,
                tiene_password: tiene_clave_privada,
                tiene_totp: tiene_2fa,
                tiene_passkey: tiene_clave_publica,
                tiene_url: tiene_host,
                tiene_app,
                tiene_notas,
            }
        }
        TipoEntrada::Wallet => {
            let wallet = entrada.datos_wallet.as_ref();
            let tiene_privada = wallet.map(|w| !w.clave_privada.trim().is_empty()).unwrap_or(false);
            let tiene_2fa = entrada.tiene_totp();
            let tiene_semilla = wallet.map(|w| !w.frase_semilla.trim().is_empty()).unwrap_or(false);
            let tiene_direccion = wallet.map(|w| !w.direccion_publica.trim().is_empty()).unwrap_or(false) || tiene_web;

            IndicadoresEntrada {
                tiene_usuario: false,
                tiene_password: tiene_privada,
                tiene_totp: tiene_2fa,
                tiene_passkey: tiene_semilla,
                tiene_url: tiene_direccion,
                tiene_app,
                tiene_notas,
            }
        }
        _ => {
            // General: Login, Passkey, Contacto, Nota, etc.
            let tiene_usuario = !entrada.usuario.trim().is_empty()
                || entrada.datos_tarjeta.as_ref().map(|t| !t.titular.trim().is_empty()).unwrap_or(false)
                || entrada.datos_cuenta_bancaria.as_ref().map(|c| !c.titular.trim().is_empty()).unwrap_or(false)
                || entrada.datos_identidad.as_ref().map(|i| !i.nombre_completo.trim().is_empty()).unwrap_or(false)
                || entrada.datos_servidor.as_ref().map(|s| !s.usuario.trim().is_empty()).unwrap_or(false);

            let tiene_contrasena = !entrada.contrasena.is_empty()
                || entrada.datos_wifi.as_ref().map(|w| !w.contrasena.trim().is_empty()).unwrap_or(false)
                || entrada.datos_tarjeta.as_ref().map(|t| !t.cvv.trim().is_empty() || !t.pin.trim().is_empty()).unwrap_or(false)
                || entrada.datos_servidor.as_ref().map(|s| !s.clave_privada.trim().is_empty()).unwrap_or(false)
                || entrada.datos_wallet.as_ref().map(|w| !w.clave_privada.trim().is_empty()).unwrap_or(false);

            let tiene_2fa = entrada.tiene_totp();

            let tiene_passkey = entrada.passkey.is_some()
                || !entrada.passkeys_secundarias.is_empty()
                || entrada.tipo == TipoEntrada::Passkey
                || entrada.datos_wallet.as_ref().map(|w| !w.frase_semilla.trim().is_empty()).unwrap_or(false)
                || entrada.datos_servidor.as_ref().map(|s| !s.clave_publica.trim().is_empty()).unwrap_or(false);

            if !tiene_web
                && (entrada.datos_servidor.as_ref().map(|s| !s.host.trim().is_empty()).unwrap_or(false)
                    || entrada.datos_wallet.as_ref().map(|w| !w.direccion_publica.trim().is_empty()).unwrap_or(false))
            {
                tiene_web = true;
            }

            IndicadoresEntrada {
                tiene_usuario,
                tiene_password: tiene_contrasena,
                tiene_totp: tiene_2fa,
                tiene_passkey,
                tiene_url: tiene_web,
                tiene_app,
                tiene_notas,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::passkey::DatosPasskey;

    #[test]
    fn test_deteccion_amazon_cinco_iconos() {
        let entrada = Entrada {
            tipo: TipoEntrada::Passkey,
            titulo: "Amazon".into(),
            usuario: "jolunavi@gmail.com".into(),
            contrasena: "secretpass".into(),
            urls: vec!["https://www.amazon.com/".into()],
            secreto_totp: Some("JBSWY3DPEHPK3PXP".into()),
            passkey: Some(DatosPasskey {
                rp_id: "amazon.com".into(),
                rp_name: "Amazon".into(),
                user_handle: "dXNlcg==".into(),
                cred_id: "cred123".into(),
                clave_privada: "privkey".into(),
                algoritmo: "ES256".into(),
                usuario: "jolunavi@gmail.com".into(),
                creada_en: 1700000000,
            }),
            ..Default::default()
        };

        let ind = detectar_indicadores(&entrada);
        assert!(ind.tiene_usuario, "Debe tener usuario");
        assert!(ind.tiene_password, "Debe tener contraseña");
        assert!(ind.tiene_totp, "Debe tener 2FA");
        assert!(ind.tiene_passkey, "Debe tener Passkey");
        assert!(ind.tiene_url, "Debe tener Web");
        assert!(!ind.tiene_app, "No tiene paquete app Android");
    }

    #[test]
    fn test_es_paquete_app() {
        assert!(es_paquete_app("android://com.whatsapp"));
        assert!(es_paquete_app("package:com.instagram.android"));
        assert!(es_paquete_app("com.amazon.mShop.android.shopping"));
        assert!(!es_paquete_app("https://www.amazon.com"));
        assert!(!es_paquete_app("github.com"));
    }
}
