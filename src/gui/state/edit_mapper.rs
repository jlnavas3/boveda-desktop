/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Mapeo bidireccional entre el modelo inmutable Rust `Entrada` y el modelo reactivo Slint `EntryEditData`.

use chrono::DateTime;

use crate::gui::EntryEditData;
use crate::models::campo_personalizado::CampoPersonalizado;
use crate::models::tipo_campo::TipoCampo;
use crate::models::tipo_entrada::TipoEntrada;
use crate::models::Entrada;

/// Convierte una entidad Entrada de Rust a la estructura reactiva EntryEditData de Slint.
pub fn entrada_a_edit_data(entrada: &Entrada) -> EntryEditData {
    let urls_secundarias = if entrada.urls.len() > 1 {
        entrada.urls[1..].join(", ")
    } else {
        String::new()
    };

    let etiquetas = entrada.etiquetas.join(", ");

    let fecha_caducidad = entrada
        .fecha_caducidad
        .and_then(|ms| {
            DateTime::from_timestamp_millis(ms).map(|dt| dt.format("%Y-%m-%d").to_string())
        })
        .unwrap_or_default();

    let mut campos_str = String::new();
    for cp in &entrada.campos_personalizados {
        if cp.es_sensible_efectivo() {
            campos_str.push_str(&format!("*{}: {}\n", cp.etiqueta, cp.valor));
        } else {
            campos_str.push_str(&format!("{}: {}\n", cp.etiqueta, cp.valor));
        }
    }

    let mut d = EntryEditData {
        id: entrada.id.as_str().into(),
        tipo: entrada.tipo.as_str().into(),
        titulo: entrada.titulo.as_str().into(),
        favorito: entrada.favorito,
        categoria_id: entrada.categorias.first().map(|c| c.as_str()).unwrap_or("").into(),
        usuario: entrada.usuario.as_str().into(),
        password: entrada.contrasena.as_str().into(),
        url: entrada.urls.first().map(|u| u.as_str()).unwrap_or("").into(),
        urls_secundarias: urls_secundarias.into(),
        notas: entrada.notas.as_str().into(),
        totp_secreto: entrada.secreto_totp.clone().unwrap_or_default().into(),
        totp_emisor: entrada.totp_emisor.as_str().into(),
        totp_periodo: entrada.totp_periodo,
        totp_digitos: entrada.totp_digitos,
        fecha_caducidad: fecha_caducidad.into(),
        etiquetas: etiquetas.into(),
        campos_personalizados: campos_str.trim().into(),
        ..Default::default()
    };

    if let Some(ref t) = entrada.datos_tarjeta {
        d.tarjeta_titular = t.titular.as_str().into();
        d.tarjeta_numero = t.numero.as_str().into();
        d.tarjeta_vencimiento = t.vencimiento.as_str().into();
        d.tarjeta_cvv = t.cvv.as_str().into();
        d.tarjeta_pin = t.pin.as_str().into();
    }
    if let Some(ref w) = entrada.datos_wifi {
        d.wifi_ssid = w.ssid.as_str().into();
        d.wifi_password = w.contrasena.as_str().into();
        d.wifi_seguridad = w.tipo_seguridad.as_str().into();
    }
    if let Some(ref s) = entrada.datos_servidor {
        d.servidor_host = s.host.as_str().into();
        d.servidor_puerto = s.puerto.as_str().into();
        d.servidor_usuario = s.usuario.as_str().into();
        d.servidor_password = s.clave_privada.as_str().into();
    }
    if let Some(ref b) = entrada.datos_cuenta_bancaria {
        d.banco_entidad = b.banco.as_str().into();
        d.banco_titular = b.titular.as_str().into();
        d.banco_numero_cuenta = b.numero_cuenta.as_str().into();
        d.banco_swift_bic = b.swift_bic.as_str().into();
    }
    if let Some(ref wl) = entrada.datos_wallet {
        d.wallet_red = wl.red.as_str().into();
        d.wallet_direccion = wl.direccion_publica.as_str().into();
        d.wallet_semilla = wl.frase_semilla.as_str().into();
    }
    if let Some(ref id) = entrada.datos_identidad {
        d.identidad_nombre = id.nombre_completo.as_str().into();
        d.identidad_tipo_doc = id.tipo_documento.as_str().into();
        d.identidad_numero = id.numero.as_str().into();
    }
    if let Some(ref c) = entrada.datos_contacto {
        d.contacto_nombre = c.nombre.as_str().into();
        d.contacto_correo = c.correo.as_str().into();
        d.contacto_telefono = if !c.telefono_secundario.is_empty() {
            c.telefono_secundario.as_str().into()
        } else {
            entrada.usuario.as_str().into()
        };
        d.contacto_empresa = c.empresa.as_str().into();
    }

    d
}

/// Convierte la estructura Slint EntryEditData al modelo fuertemente tipado Entrada de Rust,
/// preservando intactos todos los metadatos secundarios no modificables desde este formulario.
pub fn edit_data_a_entrada(data: &EntryEditData, original: Option<&Entrada>) -> Entrada {
    let ahora = chrono::Utc::now().timestamp_millis();

    let mut e = if let Some(orig) = original {
        orig.clone()
    } else {
        Entrada {
            id: if data.id.is_empty() {
                uuid::Uuid::new_v4().to_string()
            } else {
                data.id.to_string()
            },
            creada_en: ahora,
            ..Default::default()
        }
    };

    e.modificada_en = ahora;
    e.tipo = TipoEntrada::desde_str(data.tipo.as_str());
    e.titulo = data.titulo.to_string();
    e.favorito = data.favorito;
    e.usuario = data.usuario.to_string();
    e.contrasena = data.password.to_string();
    e.notas = data.notas.to_string();

    // TOTP
    let totp_sec = data.totp_secreto.trim();
    if !totp_sec.is_empty() {
        e.secreto_totp = Some(totp_sec.to_string());
    } else {
        e.secreto_totp = None;
    }
    e.totp_emisor = data.totp_emisor.trim().to_string();
    if data.totp_periodo > 0 {
        e.totp_periodo = data.totp_periodo;
    }
    if data.totp_digitos > 0 {
        e.totp_digitos = data.totp_digitos;
    }

    // Fecha de caducidad
    let cad_str = data.fecha_caducidad.trim();
    if !cad_str.is_empty() {
        if let Ok(dt) = chrono::NaiveDate::parse_from_str(cad_str, "%Y-%m-%d") {
            if let Some(naive_dt) = dt.and_hms_opt(23, 59, 59) {
                e.fecha_caducidad = Some(naive_dt.and_utc().timestamp_millis());
            }
        }
    } else {
        e.fecha_caducidad = None;
    }

    // Etiquetas
    let tags: Vec<String> = data
        .etiquetas
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    e.etiquetas = tags;

    // URLs (principal + secundarias)
    let mut urls = Vec::new();
    let url_prin = data.url.trim();
    if !url_prin.is_empty() {
        urls.push(url_prin.to_string());
    }
    for u in data.urls_secundarias.split([',', '\n']) {
        let u_trim = u.trim();
        if !u_trim.is_empty() && !urls.contains(&u_trim.to_string()) {
            urls.push(u_trim.to_string());
        }
    }
    e.urls = urls;

    // Campos personalizados
    let mut campos = Vec::new();
    for linea in data.campos_personalizados.lines() {
        let l_trim = linea.trim();
        if l_trim.is_empty() {
            continue;
        }
        let (es_sensible, resto) = if let Some(stripped) = l_trim.strip_prefix('*') {
            (true, stripped.trim())
        } else {
            (false, l_trim)
        };
        let partes: Vec<&str> = resto.splitn(2, ':').collect();
        if partes.len() == 2 {
            let etiqueta = partes[0].trim().to_string();
            let valor = partes[1].trim().to_string();
            if !etiqueta.is_empty() {
                campos.push(CampoPersonalizado {
                    id: uuid::Uuid::new_v4().to_string(),
                    etiqueta,
                    valor,
                    tipo: if es_sensible {
                        TipoCampo::Oculto
                    } else {
                        TipoCampo::Texto
                    },
                    es_sensible,
                    es_obligatorio: false,
                    formato: None,
                });
            }
        }
    }
    e.campos_personalizados = campos;

    // Categoría
    let cat_id = data.categoria_id.trim();
    if !cat_id.is_empty() && !e.categorias.contains(&cat_id.to_string()) {
        e.categorias.insert(0, cat_id.to_string());
    }

    // Mapeo de Submodelos según TipoEntrada
    match e.tipo {
        TipoEntrada::Tarjeta => {
            let mut t = e.datos_tarjeta.take().unwrap_or_default();
            t.titular = data.tarjeta_titular.to_string();
            t.numero = data.tarjeta_numero.to_string();
            t.vencimiento = data.tarjeta_vencimiento.to_string();
            t.cvv = data.tarjeta_cvv.to_string();
            t.pin = data.tarjeta_pin.to_string();
            e.datos_tarjeta = Some(t);
        }
        TipoEntrada::Wifi => {
            let mut w = e.datos_wifi.take().unwrap_or_default();
            w.ssid = data.wifi_ssid.to_string();
            w.contrasena = data.wifi_password.to_string();
            w.tipo_seguridad = if data.wifi_seguridad.trim().is_empty() {
                "WPA".to_string()
            } else {
                data.wifi_seguridad.to_string()
            };
            e.datos_wifi = Some(w);
        }
        TipoEntrada::Servidor => {
            let mut s = e.datos_servidor.take().unwrap_or_default();
            s.host = data.servidor_host.to_string();
            s.puerto = if data.servidor_puerto.trim().is_empty() {
                "22".to_string()
            } else {
                data.servidor_puerto.to_string()
            };
            s.usuario = data.servidor_usuario.to_string();
            s.clave_privada = data.servidor_password.to_string();
            e.datos_servidor = Some(s);
        }
        TipoEntrada::CuentaBancaria => {
            let mut b = e.datos_cuenta_bancaria.take().unwrap_or_default();
            b.banco = data.banco_entidad.to_string();
            b.titular = data.banco_titular.to_string();
            b.numero_cuenta = data.banco_numero_cuenta.to_string();
            b.swift_bic = data.banco_swift_bic.to_string();
            e.datos_cuenta_bancaria = Some(b);
        }
        TipoEntrada::Wallet => {
            let mut w = e.datos_wallet.take().unwrap_or_default();
            w.red = if data.wallet_red.trim().is_empty() {
                "Bitcoin".to_string()
            } else {
                data.wallet_red.to_string()
            };
            w.direccion_publica = data.wallet_direccion.to_string();
            w.frase_semilla = data.wallet_semilla.to_string();
            e.datos_wallet = Some(w);
        }
        TipoEntrada::Identidad => {
            let mut i = e.datos_identidad.take().unwrap_or_default();
            i.nombre_completo = data.identidad_nombre.to_string();
            i.tipo_documento = if data.identidad_tipo_doc.trim().is_empty() {
                "CI".to_string()
            } else {
                data.identidad_tipo_doc.to_string()
            };
            i.numero = data.identidad_numero.to_string();
            e.datos_identidad = Some(i);
        }
        TipoEntrada::Contacto => {
            let mut c = e.datos_contacto.take().unwrap_or_default();
            c.nombre = data.contacto_nombre.to_string();
            c.correo = data.contacto_correo.to_string();
            c.telefono_secundario = data.contacto_telefono.to_string();
            c.empresa = data.contacto_empresa.to_string();
            if e.usuario.is_empty() {
                e.usuario = data.contacto_telefono.to_string();
            }
            e.datos_contacto = Some(c);
        }
        TipoEntrada::Nota => {
            // Notas se almacenan directamente en e.notas
        }
        TipoEntrada::Login | TipoEntrada::Passkey => {}
    }

    e
}
