/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Mapeo bidireccional entre el modelo inmutable Rust `Entrada` y el modelo reactivo Slint `EntryEditData`.

use chrono::DateTime;

use crate::gui::EntryEditData;
use crate::models::cambio_contrasena::CambioContrasena;
use crate::models::campo_personalizado::CampoPersonalizado;
use crate::models::tipo_campo::TipoCampo;
use crate::models::tipo_entrada::TipoEntrada;
use crate::models::totp_secundario::TotpSecundario;
use crate::models::Entrada;

/// Convierte una entidad Entrada de Rust a la estructura reactiva EntryEditData de Slint.
pub fn entrada_a_edit_data(entrada: &Entrada) -> EntryEditData {
    let mut urls_web = Vec::new();
    let mut apps_vinculadas = Vec::new();
    for u in &entrada.urls {
        let u_trim = u.trim();
        if let Some(app_pkg) = u_trim.strip_prefix("android://").or_else(|| u_trim.strip_prefix("app://")) {
            apps_vinculadas.push(app_pkg.to_string());
        } else if !u_trim.contains("://") && !u_trim.contains('/') && u_trim.matches('.').count() >= 2 && u_trim.starts_with("com.") {
            apps_vinculadas.push(u_trim.to_string());
        } else if !u_trim.is_empty() {
            urls_web.push(u_trim.to_string());
        }
    }

    let url_principal = urls_web.first().cloned().unwrap_or_default();
    let urls_secundarias = if urls_web.len() > 1 {
        urls_web[1..].join(", ")
    } else {
        String::new()
    };

    let paquetes_android = apps_vinculadas.join(", ");
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
        anclado: entrada.anclado,
        ignorada_en_salud: entrada.ignorada_en_salud,
        categoria_id: entrada.categorias.first().map(|c| c.as_str()).unwrap_or("").into(),
        perfil_id: entrada.perfil_id.as_deref().unwrap_or("").into(),
        usuario: entrada.usuario.as_str().into(),
        password: entrada.contrasena.as_str().into(),
        url: url_principal.into(),
        urls_secundarias: urls_secundarias.into(),
        paquetes_android: paquetes_android.into(),
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
        d.servidor_clave_publica = s.clave_publica.as_str().into();
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
        d.wallet_clave_privada = wl.clave_privada.as_str().into();
    }
    if let Some(ref id) = entrada.datos_identidad {
        d.identidad_nombre = id.nombre_completo.as_str().into();
        d.identidad_tipo_doc = id.tipo_documento.as_str().into();
        d.identidad_numero = id.numero.as_str().into();
        d.identidad_expedicion = id.expedicion.as_str().into();
        d.identidad_caducidad = id.caducidad.as_str().into();
        d.identidad_pais = id.pais_emisor.as_str().into();
    }
    if let Some(ref c) = entrada.datos_contacto {
        d.contacto_nombre = c.nombre.as_str().into();
        d.contacto_correo = c.correo.as_str().into();
        d.contacto_telefono_principal = entrada.usuario.as_str().into();
        d.contacto_telefono = c.telefono_secundario.as_str().into();
        d.contacto_empresa = c.empresa.as_str().into();
        d.contacto_direccion = c.direccion.as_str().into();
        d.contacto_cp = c.codigo_postal.as_str().into();
        d.contacto_ciudad = c.ciudad.as_str().into();
        d.contacto_provincia = c.provincia.as_str().into();
        d.contacto_pais = c.pais.as_str().into();
    }

    d
}

/// Convierte la estructura Slint EntryEditData al modelo fuertemente tipado Entrada de Rust,
/// preservando intactos todos los metadatos secundarios y respaldando contraseñas/TOTPs modificados.
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

    // Respaldar contraseña anterior en historial_contrasenas si cambió durante la edición
    let nueva_contrasena = data.password.to_string();
    if let Some(orig) = original {
        if !orig.contrasena.is_empty() && orig.contrasena != nueva_contrasena {
            let ya_en_historial = e
                .historial_contrasenas
                .last()
                .map(|h| h.contrasena == orig.contrasena)
                .unwrap_or(false);
            if !ya_en_historial {
                e.historial_contrasenas.push(CambioContrasena {
                    contrasena: orig.contrasena.clone(),
                    cambiada_en: ahora,
                });
            }
        }
    }

    // Respaldar TOTP anterior en totps_secundarios si fue reemplazado por otro distinto
    let nuevo_totp_trim = data.totp_secreto.trim();
    if let Some(orig) = original {
        if let Some(ref sec_orig) = orig.secreto_totp {
            let sec_orig_trim = sec_orig.trim();
            if !sec_orig_trim.is_empty()
                && !nuevo_totp_trim.is_empty()
                && !sec_orig_trim.eq_ignore_ascii_case(nuevo_totp_trim)
            {
                let ya_respaldado = e
                    .totps_secundarios
                    .iter()
                    .any(|t| t.secreto.trim().eq_ignore_ascii_case(sec_orig_trim));
                if !ya_respaldado {
                    e.totps_secundarios.push(TotpSecundario {
                        id: uuid::Uuid::new_v4().to_string(),
                        secreto: sec_orig_trim.to_string(),
                        emisor: if orig.totp_emisor.is_empty() {
                            orig.titulo.clone()
                        } else {
                            orig.totp_emisor.clone()
                        },
                        digitos: if orig.totp_digitos > 0 { orig.totp_digitos } else { 6 },
                        periodo: if orig.totp_periodo > 0 { orig.totp_periodo } else { 30 },
                        algoritmo: if orig.totp_algoritmo.is_empty() {
                            "HmacSHA1".to_string()
                        } else {
                            orig.totp_algoritmo.clone()
                        },
                        agregado_en: ahora,
                    });
                }
            }
        }
    }

    e.modificada_en = ahora;
    e.tipo = TipoEntrada::desde_str(data.tipo.as_str());
    e.titulo = data.titulo.to_string();
    e.favorito = data.favorito;
    e.anclado = data.anclado;
    e.ignorada_en_salud = data.ignorada_en_salud;
    e.usuario = data.usuario.to_string();
    e.contrasena = nueva_contrasena;
    e.notas = data.notas.to_string();

    let perfil_trim = data.perfil_id.trim();
    if perfil_trim.is_empty() {
        e.perfil_id = None;
    } else {
        e.perfil_id = Some(perfil_trim.to_string());
    }

    // TOTP
    if !nuevo_totp_trim.is_empty() {
        e.secreto_totp = Some(nuevo_totp_trim.to_string());
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

    // Fecha de caducidad (soporta AAAA-MM-DD o presets relativos +30d / +90d / +180d / +365d)
    let cad_str = data.fecha_caducidad.trim();
    if let Some(dias_str) = cad_str.strip_prefix('+').and_then(|s| s.strip_suffix('d')) {
        if let Ok(dias) = dias_str.parse::<i64>() {
            e.fecha_caducidad = Some(ahora + dias * 86_400_000);
        }
    } else if !cad_str.is_empty() {
        if let Ok(dt) = chrono::NaiveDate::parse_from_str(cad_str, "%Y-%m-%d") {
            if let Some(naive_dt) = dt.and_hms_opt(23, 59, 59) {
                e.fecha_caducidad = Some(naive_dt.and_utc().timestamp_millis());
            }
        }
    } else {
        e.fecha_caducidad = None;
    }

    // Etiquetas
    e.etiquetas = data
        .etiquetas
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // URLs (principal + secundarias + paquetes Android vinculados)
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
    for pkg in data.paquetes_android.split([',', '\n']) {
        let pkg_trim = pkg.trim();
        if !pkg_trim.is_empty() {
            let formatted = if pkg_trim.starts_with("android://") || pkg_trim.starts_with("app://") {
                pkg_trim.to_string()
            } else {
                format!("android://{pkg_trim}")
            };
            if !urls.contains(&formatted) {
                urls.push(formatted);
            }
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
    if cat_id.is_empty() {
        e.categorias.clear();
    } else if !e.categorias.contains(&cat_id.to_string()) {
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
            s.clave_publica = data.servidor_clave_publica.to_string();
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
            w.clave_privada = data.wallet_clave_privada.to_string();
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
            i.expedicion = data.identidad_expedicion.to_string();
            i.caducidad = data.identidad_caducidad.to_string();
            i.pais_emisor = data.identidad_pais.to_string();
            e.datos_identidad = Some(i);
        }
        TipoEntrada::Contacto => {
            let mut c = e.datos_contacto.take().unwrap_or_default();
            c.nombre = data.contacto_nombre.to_string();
            c.correo = data.contacto_correo.to_string();
            c.telefono_secundario = data.contacto_telefono.to_string();
            c.empresa = data.contacto_empresa.to_string();
            c.direccion = data.contacto_direccion.to_string();
            c.codigo_postal = data.contacto_cp.to_string();
            c.ciudad = data.contacto_ciudad.to_string();
            c.provincia = data.contacto_provincia.to_string();
            c.pais = data.contacto_pais.to_string();
            if !data.contacto_telefono_principal.trim().is_empty() {
                e.usuario = data.contacto_telefono_principal.to_string();
            } else if e.usuario.is_empty() {
                e.usuario = data.contacto_telefono.to_string();
            }
            e.datos_contacto = Some(c);
        }
        TipoEntrada::Nota => {}
        TipoEntrada::Login | TipoEntrada::Passkey => {}
    }

    // Gancho de edición local: si el contenido cambió, incrementar revisión
    if let Some(orig) = original {
        if crate::sync::calcular_content_hash(orig) != crate::sync::calcular_content_hash(&e) {
            e.revision = orig.revision + 1;
        } else {
            e.revision = orig.revision;
        }
    } else {
        e.revision = 1;
    }

    e
}
