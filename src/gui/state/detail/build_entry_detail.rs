/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Conversor desacoplado que construye `EntryDetailData` a partir de una `Entrada` y `ContenidoBoveda`.

use std::collections::HashSet;

use slint::{ModelRc, SharedString, VecModel};

use crate::crypto::totp;
use crate::gui::{
    DetailAttachmentItem, DetailCustomFieldItem, DetailLinkItem, EntryDetailData,
    PasswordHistoryItem, SecondaryPasskeyItem, SecondaryTotpItem,
};
use crate::models::{ContenidoBoveda, Entrada};

/// Construye el modelo detallado reactivo `EntryDetailData` para la ficha de visualización.
pub fn construir_detalle_entrada(
    e: &Entrada,
    boveda: &ContenidoBoveda,
    ocultar_usuario: bool,
    ocultar_contrasena: bool,
) -> EntryDetailData {
    // 1. Categorías (todas las asignadas separadas por ", ")
    let nombres_cats: Vec<&str> = e
        .categorias
        .iter()
        .filter_map(|id| {
            boveda
                .categorias
                .iter()
                .find(|c| &c.id == id)
                .map(|c| c.nombre.as_str())
        })
        .collect();
    let cat_nombre = if nombres_cats.is_empty() {
        "Sin categoría".to_string()
    } else {
        nombres_cats.join(", ")
    };

    // 2. Perfil / Identidad asociada
    let perfil_nombre = e
        .perfil_id
        .as_ref()
        .and_then(|pid| {
            boveda
                .perfiles
                .iter()
                .find(|p| &p.id == pid)
                .map(|p| p.nombre.as_str())
        })
        .unwrap_or("");

    // 3. Evaluación de fecha de caducidad
    let (caducidad_texto, caducidad_estado) = evaluar_caducidad(e.fecha_caducidad);

    // 4. Código TOTP principal en vivo
    let seg_actual = chrono::Utc::now().timestamp() as u64;
    let totp_codigo = if let Some(ref sec) = e.secreto_totp {
        if let Ok(bytes) = totp::decodificar_base32(sec) {
            let periodo = if e.totp_periodo > 0 {
                e.totp_periodo as u64
            } else {
                30
            };
            let digitos = if e.totp_digitos > 0 {
                e.totp_digitos as usize
            } else {
                6
            };
            let restantes = totp::segundos_restantes_totp(seg_actual, periodo);
            if let Ok(cod) = totp::codigo_totp(&bytes, seg_actual, digitos, periodo) {
                format!("{} ({}s)", totp::formatear_codigo_totp(&cod), restantes)
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    // 5. Enlaces múltiples (URLs y paquetes de apps)
    let total_urls = e.urls.len();
    let enlaces_vec: Vec<DetailLinkItem> = e
        .urls
        .iter()
        .enumerate()
        .filter(|(_, u)| !u.trim().is_empty())
        .map(|(idx, u)| {
            let u_trim = u.trim();
            let es_app = u_trim.starts_with("android://")
                || u_trim.starts_with("app://")
                || (!u_trim.contains("://") && !u_trim.contains('/') && u_trim.matches('.').count() >= 2 && u_trim.starts_with("com."));
            let etiqueta = if es_app {
                if total_urls > 1 {
                    format!("Aplicación vinculada #{}", idx + 1)
                } else {
                    "Aplicación vinculada".to_string()
                }
            } else if total_urls > 1 {
                format!("URL / Sitio Web #{}", idx + 1)
            } else {
                "URL / Sitio Web".to_string()
            };
            DetailLinkItem {
                valor: SharedString::from(u_trim),
                etiqueta: SharedString::from(etiqueta),
                es_app,
            }
        })
        .collect();

    // 6. Llave de acceso principal (Passkey)
    let (
        tiene_passkey,
        passkey_rp_id,
        passkey_rp_name,
        passkey_usuario,
        passkey_fecha,
        passkey_cred_id,
        passkey_user_handle,
    ) = if let Some(ref pk) = e.passkey {
        let ts = if pk.creada_en > 0 {
            pk.creada_en
        } else {
            e.creada_en
        };
        (
            true,
            SharedString::from(&pk.rp_id),
            SharedString::from(&pk.rp_name),
            SharedString::from(&pk.usuario),
            SharedString::from(formatear_fecha_corta(ts)),
            SharedString::from(&pk.cred_id),
            SharedString::from(&pk.user_handle),
        )
    } else {
        (
            false,
            SharedString::default(),
            SharedString::default(),
            SharedString::default(),
            SharedString::default(),
            SharedString::default(),
            SharedString::default(),
        )
    };

    // 7. Credenciales alternativas: Historial de contraseñas (deduplicado y excluyendo la actual)
    let mut vistas = HashSet::new();
    let mut hist_ordenado = e.historial_contrasenas.clone();
    hist_ordenado.sort_by_key(|a| std::cmp::Reverse(a.cambiada_en));
    let historial_vec: Vec<PasswordHistoryItem> = hist_ordenado
        .into_iter()
        .filter(|c| !c.contrasena.is_empty() && c.contrasena != e.contrasena)
        .filter(|c| vistas.insert(c.contrasena.clone()))
        .map(|c| PasswordHistoryItem {
            password: SharedString::from(&c.contrasena),
            fecha: SharedString::from(formatear_fecha_corta(c.cambiada_en)),
        })
        .collect();

    // 8. Credenciales alternativas: Passkeys secundarias
    let passkeys_sec_vec: Vec<SecondaryPasskeyItem> = e
        .passkeys_secundarias
        .iter()
        .map(|pk| {
            let ts = if pk.creada_en > 0 {
                pk.creada_en
            } else {
                e.creada_en
            };
            SecondaryPasskeyItem {
                cred_id: SharedString::from(&pk.cred_id),
                rp_id: SharedString::from(&pk.rp_id),
                rp_name: SharedString::from(&pk.rp_name),
                usuario: SharedString::from(&pk.usuario),
                fecha: SharedString::from(formatear_fecha_corta(ts)),
            }
        })
        .collect();

    // 9. Credenciales alternativas: Códigos 2FA / TOTP secundarios en vivo
    let totps_sec_vec: Vec<SecondaryTotpItem> = e
        .totps_secundarios
        .iter()
        .map(|t| {
            let periodo = if t.periodo > 0 { t.periodo as u64 } else { 30 };
            let digitos = if t.digitos > 0 { t.digitos as usize } else { 6 };
            let (cod_fmt, rem) = if let Ok(bytes) = totp::decodificar_base32(&t.secreto) {
                let r = totp::segundos_restantes_totp(seg_actual, periodo) as i32;
                let c = totp::codigo_totp(&bytes, seg_actual, digitos, periodo)
                    .map(|raw| totp::formatear_codigo_totp(&raw))
                    .unwrap_or_default();
                (c, r)
            } else {
                (String::new(), 0)
            };
            SecondaryTotpItem {
                id: SharedString::from(&t.id),
                emisor: SharedString::from(&t.emisor),
                codigo: SharedString::from(cod_fmt),
                restante: rem,
            }
        })
        .collect();

    // 10. Campos personalizados
    let campos_vec: Vec<DetailCustomFieldItem> = e
        .campos_personalizados
        .iter()
        .filter(|cp| !cp.etiqueta.trim().is_empty() || !cp.valor.trim().is_empty())
        .map(|cp| DetailCustomFieldItem {
            etiqueta: SharedString::from(&cp.etiqueta),
            valor: SharedString::from(&cp.valor),
            es_sensible: cp.es_sensible_efectivo(),
        })
        .collect();

    // 11. Etiquetas
    let etiquetas_vec: Vec<SharedString> = e
        .etiquetas
        .iter()
        .filter(|t| !t.trim().is_empty())
        .map(|t| SharedString::from(t.trim()))
        .collect();

    // 12. Archivos adjuntos
    let adjuntos_vec: Vec<DetailAttachmentItem> = e
        .adjuntos
        .iter()
        .map(|adj| DetailAttachmentItem {
            id: SharedString::from(&adj.id),
            nombre: SharedString::from(&adj.nombre),
            tamano: SharedString::from(formatear_tamano_bytes(adj.tamano_bytes)),
            mime_type: SharedString::from(&adj.mime_type),
        })
        .collect();

    let _ = ocultar_contrasena;

    let mut d = EntryDetailData {
        id: SharedString::from(&e.id),
        titulo: SharedString::from(&e.titulo),
        tipo: SharedString::from(e.tipo.as_str()),
        favorito: e.favorito,
        anclado: e.anclado,
        papelera: e.eliminada_en > 0,
        creada_en: SharedString::from(formatear_fecha(e.creada_en)),
        modificada_en: SharedString::from(formatear_fecha(e.modificada_en)),
        categoria_nombre: SharedString::from(cat_nombre),
        perfil_nombre: SharedString::from(perfil_nombre),
        caducidad_texto: SharedString::from(caducidad_texto),
        caducidad_estado: SharedString::from(caducidad_estado),
        usuario: SharedString::from(if ocultar_usuario && !e.usuario.is_empty() {
            "••••••••"
        } else {
            &e.usuario
        }),
        password: SharedString::from(&e.contrasena),
        url: SharedString::from(e.urls.first().cloned().unwrap_or_default()),
        totp_codigo: SharedString::from(totp_codigo),
        enlaces: ModelRc::new(VecModel::from(enlaces_vec)),
        tiene_passkey,
        passkey_rp_id,
        passkey_rp_name,
        passkey_usuario,
        passkey_fecha,
        passkey_cred_id,
        passkey_user_handle,
        historial_passwords: ModelRc::new(VecModel::from(historial_vec)),
        passkeys_secundarias: ModelRc::new(VecModel::from(passkeys_sec_vec)),
        totps_secundarios: ModelRc::new(VecModel::from(totps_sec_vec)),
        campos_personalizados: ModelRc::new(VecModel::from(campos_vec)),
        etiquetas: ModelRc::new(VecModel::from(etiquetas_vec)),
        adjuntos: ModelRc::new(VecModel::from(adjuntos_vec)),
        notas: SharedString::from(&e.notas),
        tiene_datos: true,
        ..Default::default()
    };

    let fmt_tarjeta_4x4 = boveda
        .configuracion
        .as_ref()
        .map(|c| c.formato_tarjeta != "continuo")
        .unwrap_or(true);
    let fmt_iban_4x4 = boveda
        .configuracion
        .as_ref()
        .map(|c| c.formato_iban != "continuo")
        .unwrap_or(true);

    if let Some(ref t) = e.datos_tarjeta {
        let pan = crate::gui::state::detail::format_blocks_4x4::formatear_bloques_4x4(
            &t.numero,
            fmt_tarjeta_4x4,
        );
        d.tarjeta_numero = SharedString::from(pan);
        d.tarjeta_titular = SharedString::from(&t.titular);
        d.tarjeta_vencimiento = SharedString::from(&t.vencimiento);
        d.tarjeta_cvv = SharedString::from(&t.cvv);
        d.tarjeta_pin = SharedString::from(&t.pin);
    }
    if let Some(ref w) = e.datos_wifi {
        d.wifi_ssid = SharedString::from(&w.ssid);
        d.wifi_password = SharedString::from(&w.contrasena);
        d.wifi_seguridad = SharedString::from(&w.tipo_seguridad);
    }
    if let Some(ref s) = e.datos_servidor {
        d.servidor_host = SharedString::from(&s.host);
        d.servidor_puerto = SharedString::from(&s.puerto);
        d.servidor_usuario = SharedString::from(&s.usuario);
        d.servidor_password = SharedString::from(&s.clave_privada);
        d.servidor_clave_publica = SharedString::from(&s.clave_publica);
    }
    if let Some(ref b) = e.datos_cuenta_bancaria {
        let iban_fmt = crate::gui::state::detail::format_blocks_4x4::formatear_bloques_4x4(
            &b.numero_cuenta,
            fmt_iban_4x4,
        );
        d.banco_iban = SharedString::from(&iban_fmt);
        d.banco_cuenta = SharedString::from(&iban_fmt);
        d.banco_titular = SharedString::from(&b.titular);
        d.banco_bic = SharedString::from(&b.swift_bic);
        d.banco_entidad = SharedString::from(&b.banco);
    }
    if let Some(ref wl) = e.datos_wallet {
        d.wallet_red = SharedString::from(&wl.red);
        d.wallet_direccion = SharedString::from(&wl.direccion_publica);
        d.wallet_semilla = SharedString::from(&wl.frase_semilla);
        d.wallet_clave_privada = SharedString::from(&wl.clave_privada);
    }
    if let Some(ref id) = e.datos_identidad {
        d.identidad_nombre_completo = SharedString::from(&id.nombre_completo);
        d.identidad_tipo_doc = SharedString::from(&id.tipo_documento);
        d.identidad_numero_doc = SharedString::from(&id.numero);
        d.identidad_expedicion = SharedString::from(&id.expedicion);
        d.identidad_caducidad = SharedString::from(&id.caducidad);
        d.identidad_pais = SharedString::from(&id.pais_emisor);
    }
    if let Some(ref c) = e.datos_contacto {
        d.contacto_email = SharedString::from(&c.correo);
        d.contacto_telefono = SharedString::from(&e.usuario);
        d.contacto_telefono_secundario = SharedString::from(&c.telefono_secundario);
        d.contacto_empresa = SharedString::from(&c.empresa);
        d.contacto_direccion = SharedString::from(&c.direccion);
        d.contacto_cp = SharedString::from(&c.codigo_postal);
        d.contacto_ciudad = SharedString::from(&c.ciudad);
        d.contacto_provincia = SharedString::from(&c.provincia);
        d.contacto_pais = SharedString::from(&c.pais);
    }

    d
}

fn evaluar_caducidad(fecha_caducidad: Option<i64>) -> (String, String) {
    let ts = match fecha_caducidad {
        Some(v) if v > 0 => v,
        _ => return (String::new(), String::new()),
    };
    let ahora_ms = chrono::Utc::now().timestamp_millis();
    let diff_dias = (ts - ahora_ms) / (1000 * 60 * 60 * 24);
    let fecha_str = formatear_fecha_corta(ts);

    if ts < ahora_ms {
        (
            format!("Expirada ({fecha_str})"),
            "EXPIRADA".to_string(),
        )
    } else if diff_dias <= 15 {
        (
            format!("Expira en {diff_dias}d ({fecha_str})"),
            "POR_EXPIRAR".to_string(),
        )
    } else {
        (
            format!("Vence: {fecha_str}"),
            "VIGENTE".to_string(),
        )
    }
}

fn formatear_fecha(timestamp_ms: i64) -> String {
    if timestamp_ms <= 0 {
        return "Desconocida".to_string();
    }
    chrono::DateTime::from_timestamp_millis(timestamp_ms)
        .map(|dt| dt.format("%d/%m/%Y %H:%M").to_string())
        .unwrap_or_else(|| "Fecha inválida".to_string())
}

fn formatear_fecha_corta(timestamp_ms: i64) -> String {
    if timestamp_ms <= 0 {
        return String::new();
    }
    chrono::DateTime::from_timestamp_millis(timestamp_ms)
        .map(|dt| dt.format("%d/%m/%Y").to_string())
        .unwrap_or_default()
}

fn formatear_tamano_bytes(bytes: i64) -> String {
    if bytes <= 0 {
        return "0 B".to_string();
    }
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
