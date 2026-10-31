/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Motor de filtrado reactivo, ordenación, resúmenes e indexación para la lista de cuentas.

use slint::SharedString;

use crate::crypto::totp;
use crate::gui::service_grouper::extraer_nombre_servicio;
use crate::gui::state::indicator_detector::detectar_indicadores;
use crate::gui::{DropdownOption, EntryDetailData, EntryOverview, FilterItem};
use crate::models::{ContenidoBoveda, Entrada, TipoEntrada};

/// Genera un subtítulo informativo contextualizado según el tipo de credencial.
pub fn subtitulo_segun_tipo(e: &Entrada) -> String {
    match e.tipo {
        TipoEntrada::Login => {
            if !e.usuario.is_empty() {
                e.usuario.clone()
            } else if let Some(url) = e.urls.first() {
                url.clone()
            } else {
                String::new()
            }
        }
        TipoEntrada::Tarjeta => {
            if let Some(ref t) = e.datos_tarjeta {
                if t.numero.len() >= 4 {
                    format!("•••• {}", &t.numero[t.numero.len() - 4..])
                } else if !t.titular.is_empty() {
                    t.titular.clone()
                } else {
                    "Tarjeta Bancaria".to_string()
                }
            } else {
                "Tarjeta".to_string()
            }
        }
        TipoEntrada::Wifi => {
            if let Some(ref w) = e.datos_wifi {
                format!("SSID: {}", w.ssid)
            } else {
                "Red Wi-Fi".to_string()
            }
        }
        TipoEntrada::Servidor => {
            if let Some(ref s) = e.datos_servidor {
                if !s.usuario.is_empty() && !s.host.is_empty() {
                    format!("{}@{}", s.usuario, s.host)
                } else if !s.host.is_empty() {
                    s.host.clone()
                } else {
                    "Servidor SSH".to_string()
                }
            } else {
                "Servidor".to_string()
            }
        }
        TipoEntrada::CuentaBancaria => {
            if let Some(ref b) = e.datos_cuenta_bancaria {
                if !b.banco.is_empty() {
                    b.banco.clone()
                } else if !b.numero_cuenta.is_empty() {
                    b.numero_cuenta.clone()
                } else {
                    "Cuenta Bancaria".to_string()
                }
            } else {
                "Banco".to_string()
            }
        }
        TipoEntrada::Wallet => {
            if let Some(ref wl) = e.datos_wallet {
                format!("Red: {}", wl.red)
            } else {
                "Cripto Wallet".to_string()
            }
        }
        TipoEntrada::Identidad => {
            if let Some(ref id) = e.datos_identidad {
                if !id.numero.is_empty() {
                    format!("{}: {}", id.tipo_documento, id.numero)
                } else {
                    id.nombre_completo.clone()
                }
            } else {
                "Documento".to_string()
            }
        }
        TipoEntrada::Contacto => {
            if let Some(ref c) = e.datos_contacto {
                if !c.correo.is_empty() {
                    c.correo.clone()
                } else if !c.telefono_secundario.is_empty() {
                    c.telefono_secundario.clone()
                } else if !e.usuario.is_empty() {
                    e.usuario.clone()
                } else {
                    c.empresa.clone()
                }
            } else {
                "Contacto".to_string()
            }
        }
        TipoEntrada::Nota => {
            if !e.notas.is_empty() {
                let primera_linea = e.notas.lines().next().unwrap_or("");
                if primera_linea.len() > 30 {
                    format!("{}...", &primera_linea[..30])
                } else {
                    primera_linea.to_string()
                }
            } else {
                "Nota segura".to_string()
            }
        }
        TipoEntrada::Passkey => e.usuario.clone(),
    }
}

/// Filtra y mapea las entradas activas para la lista central.
pub fn filtrar_entradas(
    boveda: &ContenidoBoveda,
    filtro_tipo: &str,
    filtro_categoria: &str,
    filtro_perfil: &str,
    filtro_etiqueta: &str,
    termino_busqueda: &str,
    fijar_anclados_arriba: bool,
) -> Vec<EntryOverview> {
    let termino = termino_busqueda.trim().to_lowercase();
    let en_papelera = filtro_tipo == "PAPELERA";

    let mut filtradas: Vec<&Entrada> = boveda
        .entradas
        .iter()
        .filter(|e| if en_papelera { e.eliminada_en > 0 } else { e.eliminada_en == 0 })
        .filter(|e| {
            if filtro_tipo == "TODOS" || filtro_tipo == "PAPELERA" {
                true
            } else {
                e.tipo.as_str().eq_ignore_ascii_case(filtro_tipo)
            }
        })
        .filter(|e| {
            if filtro_categoria == "TODAS" {
                true
            } else {
                e.categorias.contains(&filtro_categoria.to_string())
            }
        })
        .filter(|e| {
            if filtro_perfil == "TODOS" {
                true
            } else {
                e.perfil_id.as_deref() == Some(filtro_perfil)
            }
        })
        .filter(|e| {
            if filtro_etiqueta == "TODAS" {
                true
            } else {
                e.etiquetas.iter().any(|t| t.eq_ignore_ascii_case(filtro_etiqueta))
            }
        })
        .filter(|e| {
            if termino.is_empty() {
                return true;
            }
            e.titulo.to_lowercase().contains(&termino)
                || e.usuario.to_lowercase().contains(&termino)
                || e.urls.iter().any(|u| u.to_lowercase().contains(&termino))
                || e.notas.to_lowercase().contains(&termino)
                || e.etiquetas.iter().any(|t| t.to_lowercase().contains(&termino))
        })
        .collect();

    // Ordenación
    filtradas.sort_by(|a, b| {
        if fijar_anclados_arriba && a.anclado != b.anclado {
            return b.anclado.cmp(&a.anclado);
        }
        a.titulo.to_lowercase().cmp(&b.titulo.to_lowercase())
    });

    filtradas
        .into_iter()
        .map(|e| {
            let subtitulo = subtitulo_segun_tipo(e);
            let primera_categoria = e.categorias.first().cloned().unwrap_or_default();
            let ind = detectar_indicadores(e);

            let ahora = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);

            let (totp_codigo, totp_restante) = if let Some(ref sec) = e.secreto_totp {
                if !sec.trim().is_empty() {
                    let digitos = if e.totp_digitos > 0 { e.totp_digitos as usize } else { 6 };
                    let periodo = if e.totp_periodo > 0 { e.totp_periodo as u64 } else { 30 };
                    let cod = totp::codigo_totp_desde_base32(sec, ahora, digitos, periodo)
                        .map(|c| totp::formatear_codigo_totp(&c))
                        .unwrap_or_default();
                    let rem = totp::segundos_restantes_totp(ahora, periodo) as i32;
                    (cod, rem)
                } else {
                    (String::new(), 0)
                }
            } else {
                (String::new(), 0)
            };

            let primer_url = e.urls.first().map(|s| s.as_str()).unwrap_or("");
            let servicio = extraer_nombre_servicio(primer_url, &e.titulo);

            EntryOverview {
                id: SharedString::from(&e.id),
                titulo: SharedString::from(&e.titulo),
                subtitulo: SharedString::from(subtitulo),
                tipo: SharedString::from(e.tipo.as_str()),
                favorito: e.favorito,
                anclado: e.anclado,
                categoria_id: SharedString::from(primera_categoria),
                tiene_usuario: ind.tiene_usuario,
                tiene_password: ind.tiene_password,
                tiene_totp: ind.tiene_totp,
                tiene_passkey: ind.tiene_passkey,
                tiene_url: ind.tiene_url,
                tiene_app: ind.tiene_app,
                tiene_notas: ind.tiene_notas,
                totp_codigo: SharedString::from(totp_codigo),
                totp_restante,
                servicio: SharedString::from(servicio),
                es_cabecera_grupo: false,
                clave_grupo: SharedString::from(""),
                grupo_conteo: 0,
                grupo_expandido: false,
                es_hijo_grupo: false,
            }
        })
        .collect()
}

/// Encuentra el ID de la primera entrada que comienza por la letra dada (para salto Niagara A-Z).
pub fn encontrar_id_por_letra(entradas: &[EntryOverview], letra: char) -> Option<String> {
    let target = letra.to_ascii_lowercase();
    entradas.iter().find_map(|e| {
        let t = e.titulo.chars().next()?;
        if t.to_ascii_lowercase() == target {
            Some(e.id.to_string())
        } else {
            None
        }
    })
}

/// Encuentra el índice (0-based) de la primera entrada que comienza por la letra dada (para scroll en lista).
pub fn encontrar_indice_por_letra(entradas: &[EntryOverview], letra: char) -> Option<usize> {
    let target = letra.to_ascii_lowercase();
    entradas.iter().position(|e| {
        e.titulo
            .chars()
            .next()
            .map(|c| c.to_ascii_lowercase() == target)
            .unwrap_or(false)
    })
}

/// Genera la lista de categorías con conteos reactivos para el panel lateral.
pub fn calcular_resumen_categorias(boveda: &ContenidoBoveda) -> Vec<FilterItem> {
    boveda
        .categorias
        .iter()
        .map(|cat| {
            let conteo = boveda
                .entradas
                .iter()
                .filter(|e| e.eliminada_en == 0 && e.categorias.contains(&cat.id))
                .count() as i32;

            FilterItem {
                id: SharedString::from(&cat.id),
                nombre: SharedString::from(&cat.nombre),
                icono: SharedString::from(if cat.icono.is_empty() { "📁" } else { &cat.icono }),
                conteo,
            }
        })
        .collect()
}

/// Genera la lista de tipos de entrada con sus conteos reactivos.
pub fn calcular_resumen_tipos(boveda: &ContenidoBoveda) -> Vec<FilterItem> {
    let entradas_activas: Vec<&Entrada> = boveda.entradas.iter().filter(|e| e.eliminada_en == 0).collect();
    let en_papelera = boveda.entradas.iter().filter(|e| e.eliminada_en > 0).count() as i32;

    let tipos = [
        ("TODOS", "Todas las entradas", "📋", entradas_activas.len() as i32),
        ("LOGIN", "Contraseñas", "🔑", entradas_activas.iter().filter(|e| e.tipo == TipoEntrada::Login).count() as i32),
        ("TARJETA", "Tarjetas", "💳", entradas_activas.iter().filter(|e| e.tipo == TipoEntrada::Tarjeta).count() as i32),
        ("CUENTA_BANCARIA", "Cuentas Bancarias", "🏦", entradas_activas.iter().filter(|e| e.tipo == TipoEntrada::CuentaBancaria).count() as i32),
        ("SERVIDOR", "Servidores / SSH", "🖥", entradas_activas.iter().filter(|e| e.tipo == TipoEntrada::Servidor).count() as i32),
        ("WIFI", "Redes Wi-Fi", "📶", entradas_activas.iter().filter(|e| e.tipo == TipoEntrada::Wifi).count() as i32),
        ("WALLET", "Cripto Wallets", "🪙", entradas_activas.iter().filter(|e| e.tipo == TipoEntrada::Wallet).count() as i32),
        ("IDENTIDAD", "Documentos", "🪪", entradas_activas.iter().filter(|e| e.tipo == TipoEntrada::Identidad).count() as i32),
        ("CONTACTO", "Contactos", "👤", entradas_activas.iter().filter(|e| e.tipo == TipoEntrada::Contacto).count() as i32),
        ("NOTA", "Notas Seguras", "📝", entradas_activas.iter().filter(|e| e.tipo == TipoEntrada::Nota).count() as i32),
        ("PAPELERA", "Papelera", "🗑", en_papelera),
    ];

    tipos
        .iter()
        .map(|(id, nombre, icono, conteo)| FilterItem {
            id: SharedString::from(*id),
            nombre: SharedString::from(*nombre),
            icono: SharedString::from(*icono),
            conteo: *conteo,
        })
        .collect()
}

/// Construye el modelo detallado reactivo EntryDetailData para la ficha de visualización.
pub fn construir_detalle_entrada(
    e: &Entrada,
    boveda: &ContenidoBoveda,
    ocultar_usuario: bool,
    ocultar_contrasena: bool,
) -> EntryDetailData {
    let cat_nombre = e.categorias.first().and_then(|id| {
        boveda.categorias.iter().find(|c| &c.id == id).map(|c| c.nombre.as_str())
    }).unwrap_or("Sin categoría");

    let totp_codigo = if let Some(ref sec) = e.secreto_totp {
        if let Ok(bytes) = totp::decodificar_base32(sec) {
            let seg_actual = chrono::Utc::now().timestamp() as u64;
            let periodo = if e.totp_periodo > 0 { e.totp_periodo as u64 } else { 30 };
            let digitos = if e.totp_digitos > 0 { e.totp_digitos as usize } else { 6 };
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

    let mut d = EntryDetailData {
        id: SharedString::from(&e.id),
        titulo: SharedString::from(&e.titulo),
        tipo: SharedString::from(e.tipo.as_str()),
        favorito: e.favorito,
        papelera: e.eliminada_en > 0,
        creada_en: SharedString::from(formatear_fecha(e.creada_en)),
        modificada_en: SharedString::from(formatear_fecha(e.modificada_en)),
        categoria_nombre: SharedString::from(cat_nombre),
        perfil_nombre: SharedString::from(""),
        usuario: SharedString::from(if ocultar_usuario { "••••••••" } else { &e.usuario }),
        password: SharedString::from(if ocultar_contrasena { "••••••••" } else { &e.contrasena }),
        url: SharedString::from(e.urls.first().cloned().unwrap_or_default()),
        totp_codigo: SharedString::from(totp_codigo),
        notas: SharedString::from(&e.notas),
        tiene_datos: true,
        ..Default::default()
    };

    if let Some(ref t) = e.datos_tarjeta {
        d.tarjeta_numero = SharedString::from(&t.numero);
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
    }
    if let Some(ref b) = e.datos_cuenta_bancaria {
        d.banco_iban = SharedString::from(&b.numero_cuenta);
        d.banco_cuenta = SharedString::from(&b.numero_cuenta);
        d.banco_bic = SharedString::from(&b.swift_bic);
        d.banco_entidad = SharedString::from(&b.banco);
    }
    if let Some(ref wl) = e.datos_wallet {
        d.wallet_red = SharedString::from(&wl.red);
        d.wallet_direccion = SharedString::from(&wl.direccion_publica);
        d.wallet_semilla = SharedString::from(&wl.frase_semilla);
    }
    if let Some(ref id) = e.datos_identidad {
        d.identidad_nombre_completo = SharedString::from(&id.nombre_completo);
        d.identidad_tipo_doc = SharedString::from(&id.tipo_documento);
        d.identidad_numero_doc = SharedString::from(&id.numero);
    }
    if let Some(ref c) = e.datos_contacto {
        d.contacto_email = SharedString::from(&c.correo);
        d.contacto_telefono = SharedString::from(if !c.telefono_secundario.is_empty() {
            &c.telefono_secundario
        } else {
            &e.usuario
        });
        d.contacto_empresa = SharedString::from(&c.empresa);
    }

    d
}

fn formatear_fecha(timestamp_ms: i64) -> String {
    if timestamp_ms <= 0 {
        return "Desconocida".to_string();
    }
    chrono::DateTime::from_timestamp_millis(timestamp_ms)
        .map(|dt| dt.format("%d/%m/%Y %H:%M").to_string())
        .unwrap_or_else(|| "Fecha inválida".to_string())
}

/// Genera la lista de perfiles / identidades con conteos para el filtro rápido.
pub fn calcular_resumen_perfiles(boveda: &ContenidoBoveda) -> Vec<DropdownOption> {
    let mut lista = vec![DropdownOption {
        id: SharedString::from("TODOS"),
        nombre: SharedString::from("Todos los perfiles"),
        icono: SharedString::from("👤"),
        conteo: boveda.entradas.iter().filter(|e| e.eliminada_en == 0).count() as i32,
    }];

    for p in &boveda.perfiles {
        let conteo = boveda
            .entradas
            .iter()
            .filter(|e| e.eliminada_en == 0 && e.perfil_id.as_deref() == Some(&p.id))
            .count() as i32;

        lista.push(DropdownOption {
            id: SharedString::from(&p.id),
            nombre: SharedString::from(&p.nombre),
            icono: SharedString::from(if p.icono.is_empty() { "👤" } else { &p.icono }),
            conteo,
        });
    }

    lista
}

/// Genera la lista de etiquetas únicas presentes en la bóveda con conteos.
pub fn calcular_resumen_etiquetas(boveda: &ContenidoBoveda) -> Vec<DropdownOption> {
    let mut mapa: std::collections::BTreeMap<String, i32> = std::collections::BTreeMap::new();
    for e in &boveda.entradas {
        if e.eliminada_en == 0 {
            for t in &e.etiquetas {
                let clean = t.trim().to_string();
                if !clean.is_empty() {
                    *mapa.entry(clean).or_insert(0) += 1;
                }
            }
        }
    }

    let mut lista = vec![DropdownOption {
        id: SharedString::from("TODAS"),
        nombre: SharedString::from("Todas las etiquetas"),
        icono: SharedString::from("🏷"),
        conteo: boveda.entradas.iter().filter(|e| e.eliminada_en == 0).count() as i32,
    }];

    for (tag, conteo) in mapa {
        lista.push(DropdownOption {
            id: SharedString::from(&tag),
            nombre: SharedString::from(&tag),
            icono: SharedString::from("🏷"),
            conteo,
        });
    }

    lista
}
