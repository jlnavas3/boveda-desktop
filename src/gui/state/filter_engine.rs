/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Motor de filtrado reactivo, ordenación, resúmenes e indexación para la lista de cuentas.

use slint::SharedString;

use crate::crypto::totp;
use crate::gui::profiles::{es_mismo_documento, resolver_perfil_para_entrada};
use crate::gui::service_grouper::extraer_nombre_servicio;
use crate::gui::state::indicator_detector::detectar_indicadores;
pub use crate::gui::state::detail::construir_detalle_entrada;
use crate::gui::{DropdownOption, EntryOverview, FilterItem};
use crate::models::{ContenidoBoveda, Entrada, TipoEntrada};

fn parse_hex_color(hex: &str) -> slint::Color {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() == 6 {
        let r = u8::from_str_radix(&clean[0..2], 16).unwrap_or(59);
        let g = u8::from_str_radix(&clean[2..4], 16).unwrap_or(130);
        let b = u8::from_str_radix(&clean[4..6], 16).unwrap_or(246);
        slint::Color::from_argb_u8(255, r, g, b)
    } else {
        slint::Color::from_argb_u8(255, 59, 130, 246)
    }
}

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

    // Si se consulta la papelera, unificamos entradas activas marcadas como eliminadas con boveda.papelera
    let fuentes_entradas: Vec<&Entrada> = if en_papelera {
        boveda
            .papelera
            .iter()
            .chain(boveda.entradas.iter().filter(|e| e.eliminada_en > 0))
            .collect()
    } else {
        boveda.entradas.iter().filter(|e| e.eliminada_en == 0).collect()
    };

    let mut filtradas: Vec<&Entrada> = fuentes_entradas
        .into_iter()
        .filter(|e| {
            match filtro_tipo {
                "TODOS" | "PAPELERA" => true,
                "FAVORITOS" => e.favorito,
                "TOTP" => {
                    !e.secreto_totp.as_deref().unwrap_or("").trim().is_empty()
                        || !e.totps_secundarios.is_empty()
                }
                "PASSKEY" | "PASSKEYS" => e.tipo == TipoEntrada::Passkey,
                _ => e.tipo.as_str().eq_ignore_ascii_case(filtro_tipo),
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
                resolver_perfil_para_entrada(e, &boveda.perfiles)
                    .map(|p| p.id == filtro_perfil)
                    .unwrap_or(false)
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

    // Ordenación real desacoplada según el criterio configurado en la bóveda
    let criterio_orden = boveda
        .configuracion
        .as_ref()
        .map(|c| c.criterio_ordenacion.as_str())
        .unwrap_or("NOMBRE_AZ");
    crate::gui::state::sorting::ordenar_entradas_referencias(
        &mut filtradas,
        criterio_orden,
        fijar_anclados_arriba,
    );

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

            let perfil_asociado = resolver_perfil_para_entrada(e, &boveda.perfiles);
            let mut mostrar_chip_perfil = false;
            let mut nombre_perfil = SharedString::default();
            let mut color_perfil = slint::Color::from_argb_u8(255, 59, 130, 246);
            let mut es_documento_perfil = false;

            if let Some(p) = perfil_asociado {
                let u_norm = e.usuario.trim().to_lowercase();
                let coincide_correo = !u_norm.is_empty()
                    && ((!p.correo_principal.is_empty()
                        && p.correo_principal.trim().to_lowercase() == u_norm)
                        || p.correos_secundarios
                            .iter()
                            .any(|c| c.trim().to_lowercase() == u_norm));

                let coincide_doc = match &p.numero_documento {
                    Some(doc) if !doc.trim().is_empty() => {
                        (!u_norm.is_empty() && es_mismo_documento(doc, &u_norm))
                            || e.campos_personalizados.iter().any(|cp| {
                                !cp.valor.trim().is_empty() && es_mismo_documento(doc, &cp.valor)
                            })
                    }
                    _ => false,
                };

                if coincide_correo || coincide_doc {
                    mostrar_chip_perfil = true;
                    nombre_perfil = SharedString::from(&p.nombre);
                    color_perfil = parse_hex_color(p.color_hex.as_deref().unwrap_or("#3B82F6"));
                    es_documento_perfil = coincide_doc;
                }
            }

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
                es_ultimo_hijo_grupo: false,
                seleccionada_lote: false,
                mostrar_chip_perfil,
                nombre_perfil,
                color_perfil,
                es_documento_perfil,
            }
        })
        .collect()
}

fn normalizar_inicial_alfabetica(c: char) -> char {
    let lower = c.to_lowercase().next().unwrap_or(c);
    match lower {
        'á' | 'à' | 'ä' | 'â' => 'a',
        'é' | 'è' | 'ë' | 'ê' => 'e',
        'í' | 'ì' | 'ï' | 'î' => 'i',
        'ó' | 'ò' | 'ö' | 'ô' => 'o',
        'ú' | 'ù' | 'ü' | 'û' => 'u',
        other => other,
    }
}

fn coincide_con_letra(titulo: &str, target: char) -> bool {
    titulo
        .trim()
        .chars()
        .next()
        .map(|c| normalizar_inicial_alfabetica(c) == target)
        .unwrap_or(false)
}

/// Encuentra el ID de la primera entrada o cabecera de grupo que comienza por la letra dada (para salto Niagara A-Z).
pub fn encontrar_id_por_letra(entradas: &[EntryOverview], letra: char) -> Option<String> {
    let target = normalizar_inicial_alfabetica(letra);
    entradas
        .iter()
        .find(|e| !e.anclado && !e.es_hijo_grupo && coincide_con_letra(&e.titulo, target))
        .or_else(|| entradas.iter().find(|e| coincide_con_letra(&e.titulo, target)))
        .map(|e| e.id.to_string())
}

/// Encuentra el índice (0-based) de la primera entrada o cabecera de grupo que comienza por la letra dada (para scroll en lista).
pub fn encontrar_indice_por_letra(entradas: &[EntryOverview], letra: char) -> Option<usize> {
    let target = normalizar_inicial_alfabetica(letra);
    entradas
        .iter()
        .position(|e| !e.anclado && !e.es_hijo_grupo && coincide_con_letra(&e.titulo, target))
        .or_else(|| entradas.iter().position(|e| coincide_con_letra(&e.titulo, target)))
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
    let en_papelera = (boveda.papelera.len()
        + boveda.entradas.iter().filter(|e| e.eliminada_en > 0).count()) as i32;

    let tipos = [
        ("TODOS", "Todas las entradas", "📋", entradas_activas.len() as i32),
        ("LOGIN", "Contraseñas", "🔑", entradas_activas.iter().filter(|e| e.tipo == TipoEntrada::Login).count() as i32),
        ("PASSKEY", "Llaves de acceso", "🪪", entradas_activas.iter().filter(|e| e.tipo == TipoEntrada::Passkey).count() as i32),
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

/// Genera la lista de opciones de Tipos de Entrada para la FilterBar superior (solo incluye tipos que existen en las entradas).
pub fn calcular_opciones_tipos_dropdown(boveda: &ContenidoBoveda) -> Vec<DropdownOption> {
    let resumen = calcular_resumen_tipos(boveda);
    resumen
        .into_iter()
        .filter(|item| item.id.as_str() == "TODOS" || (item.id.as_str() != "PAPELERA" && item.conteo > 0))
        .map(|item| DropdownOption {
            id: item.id,
            nombre: item.nombre,
            icono: item.icono,
            conteo: item.conteo,
        })
        .collect()
}

/// Genera la lista de opciones de Categorías para la FilterBar superior (solo incluye categorías asignadas a alguna entrada activa).
pub fn calcular_opciones_categorias_dropdown(boveda: &ContenidoBoveda) -> Vec<DropdownOption> {
    let resumen = calcular_resumen_categorias(boveda);
    let total_con_categoria: i32 = resumen.iter().map(|c| c.conteo).sum();
    let mut lista = vec![DropdownOption {
        id: SharedString::from("TODAS"),
        nombre: SharedString::from("Todas las categorías"),
        icono: SharedString::from("📁"),
        conteo: total_con_categoria,
    }];

    for cat in resumen {
        if cat.conteo > 0 {
            lista.push(DropdownOption {
                id: cat.id,
                nombre: cat.nombre,
                icono: cat.icono,
                conteo: cat.conteo,
            });
        }
    }

    lista
}

/// Genera la lista de perfiles / identidades con conteos para el filtro rápido (solo incluye perfiles presentes en entradas activas).
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
            .filter(|e| {
                e.eliminada_en == 0
                    && resolver_perfil_para_entrada(e, &boveda.perfiles)
                        .map(|res| res.id == p.id)
                        .unwrap_or(false)
            })
            .count() as i32;

        if conteo > 0 {
            lista.push(DropdownOption {
                id: SharedString::from(&p.id),
                nombre: SharedString::from(&p.nombre),
                icono: SharedString::from(if p.icono.is_empty() { "👤" } else { &p.icono }),
                conteo,
            });
        }
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
