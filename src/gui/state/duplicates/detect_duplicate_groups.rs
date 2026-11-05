/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Analizador y agrupador canónico de entradas duplicadas según servicio, usuario y credenciales.
//! Réplica fiel del pipeline de 6 fases de `AnalizadorDuplicados.kt` de Android.

use std::collections::{HashMap, HashSet};
use slint::SharedString;

use crate::gui::domain::service_extractor::extraer_nombre_servicio_canonico;
use crate::gui::state::duplicates::duplicate_filter_kind::FiltroDuplicados;
use crate::gui::state::duplicates::duplicate_normalizer::{
    es_app_android_entrada, normalizar_servicio_duplicados, normalizar_usuario_duplicados,
};
use crate::gui::state::duplicates::native_duplicate_key::{
    es_tipo_especializado_duplicados, extraer_clave_duplicado_nativo,
};
use crate::gui::DuplicateGroupItem;
use crate::models::{ContenidoBoveda, Entrada};

struct GrupoDetectado<'a> {
    tipo_duplicado: &'static str,
    etiqueta_tipo: &'static str,
    clave_visual: String,
    entradas: Vec<&'a Entrada>,
    es_app_android: bool,
}

pub fn detectar_grupos_duplicados(
    boveda: &ContenidoBoveda,
    filtro_str: &str,
) -> Vec<DuplicateGroupItem> {
    let filtro = FiltroDuplicados::desde_str(filtro_str);
    let activas: Vec<&Entrada> = boveda.entradas.iter().filter(|e| e.eliminada_en == 0).collect();

    let mut resultado_grupos: Vec<GrupoDetectado> = Vec::new();
    let mut ids_procesados: HashSet<&str> = HashSet::new();

    // 0. Detección para tipos nativos especializados (Wi-Fi, Tarjeta, Cuenta, Documento, Servidor, Wallet)
    let mut grupos_nativos: HashMap<String, Vec<&Entrada>> = HashMap::new();
    let mut etiquetas_nativas: HashMap<String, String> = HashMap::new();

    for e in &activas {
        if let Some(clave) = extraer_clave_duplicado_nativo(e) {
            etiquetas_nativas.entry(clave.identificador.clone()).or_insert(clave.etiqueta_visual);
            grupos_nativos.entry(clave.identificador).or_default().push(e);
        }
    }

    for (identificador, lista) in grupos_nativos {
        if lista.len() > 1 {
            let etiqueta = etiquetas_nativas.get(&identificador).cloned().unwrap_or(identificador);
            let todas_misma_clave = lista.iter().all(|m| m.contrasena == lista[0].contrasena);
            let (tipo, etiq) = if todas_misma_clave {
                ("IDENTICOS", "Credenciales idénticas")
            } else {
                ("MISMA_CUENTA", "Mismo identificador con distinta clave")
            };
            for e in &lista {
                ids_procesados.insert(e.id.as_str());
            }
            resultado_grupos.push(GrupoDetectado {
                tipo_duplicado: tipo,
                etiqueta_tipo: etiq,
                clave_visual: etiqueta,
                entradas: lista,
                es_app_android: false,
            });
        }
    }

    // Candidatas generales (excluyen tipos especializados no coincidentes para no agruparlos erróneamente)
    let candidatas_generales: Vec<&Entrada> = activas
        .iter()
        .copied()
        .filter(|e| !ids_procesados.contains(e.id.as_str()) && !es_tipo_especializado_duplicados(e.tipo))
        .collect();

    // 1. Detección de copias idénticas exactas (mismo servicio, usuario, contraseña, TOTP y Passkey)
    let mut grupos_identicos: HashMap<String, Vec<&Entrada>> = HashMap::new();
    for e in &candidatas_generales {
        let serv = normalizar_servicio_duplicados(e);
        let usr = e.usuario.trim().to_lowercase();
        let pwd = &e.contrasena;
        let totp = e.secreto_totp.as_deref().unwrap_or("").replace(' ', "").to_uppercase();
        let pk = e.passkey.as_ref().map(|p| p.cred_id.trim()).unwrap_or("");
        let clave = format!("{serv}|||{usr}|||{pwd}|||{totp}|||{pk}");
        grupos_identicos.entry(clave).or_default().push(e);
    }

    for (_clave, lista) in grupos_identicos {
        if lista.len() > 1 {
            let clave_vis = armar_clave_visual(lista[0]);
            let es_app = lista.iter().any(|e| es_app_android_entrada(e));
            for e in &lista {
                ids_procesados.insert(e.id.as_str());
            }
            resultado_grupos.push(GrupoDetectado {
                tipo_duplicado: "IDENTICOS",
                etiqueta_tipo: "Credenciales idénticas",
                clave_visual: clave_vis,
                entradas: lista,
                es_app_android: es_app,
            });
        }
    }

    // 2. Detección de duplicados de Passkey (mismo credId o mismo rpId + usuario)
    let restantes_passkey: Vec<&Entrada> = candidatas_generales
        .iter()
        .copied()
        .filter(|e| !ids_procesados.contains(e.id.as_str()) && e.passkey.is_some())
        .collect();

    let mut grupos_passkey: HashMap<String, Vec<&Entrada>> = HashMap::new();
    for e in &restantes_passkey {
        let pk = e.passkey.as_ref().unwrap();
        let clave = if !pk.cred_id.trim().is_empty() {
            format!("credId|||{}", pk.cred_id.trim())
        } else {
            let serv = if !pk.rp_id.trim().is_empty() {
                pk.rp_id.trim().to_lowercase()
            } else {
                normalizar_servicio_duplicados(e)
            };
            let usr = if !pk.usuario.trim().is_empty() {
                pk.usuario.trim().to_lowercase()
            } else {
                e.usuario.trim().to_lowercase()
            };
            format!("rpId|||{serv}|||{usr}")
        };
        grupos_passkey.entry(clave).or_default().push(e);
    }

    for (_clave, lista) in grupos_passkey {
        if lista.len() > 1 {
            let clave_vis = armar_clave_visual(lista[0]);
            let es_app = lista.iter().any(|e| es_app_android_entrada(e));
            for e in &lista {
                ids_procesados.insert(e.id.as_str());
            }
            resultado_grupos.push(GrupoDetectado {
                tipo_duplicado: "PASSKEY",
                etiqueta_tipo: "Llaves de acceso duplicadas",
                clave_visual: clave_vis,
                entradas: lista,
                es_app_android: es_app,
            });
        }
    }

    // 3. Detección de duplicados de TOTP (mismo secreto)
    let restantes_totp: Vec<&Entrada> = candidatas_generales
        .iter()
        .copied()
        .filter(|e| {
            !ids_procesados.contains(e.id.as_str())
                && !e.secreto_totp.as_deref().unwrap_or("").trim().is_empty()
        })
        .collect();

    let mut grupos_totp: HashMap<String, Vec<&Entrada>> = HashMap::new();
    for e in &restantes_totp {
        let sec = e.secreto_totp.as_deref().unwrap().replace(' ', "").trim().to_uppercase();
        grupos_totp.entry(sec).or_default().push(e);
    }

    for (_clave, lista) in grupos_totp {
        if lista.len() > 1 {
            let clave_vis = armar_clave_visual(lista[0]);
            let es_app = lista.iter().any(|e| es_app_android_entrada(e));
            for e in &lista {
                ids_procesados.insert(e.id.as_str());
            }
            resultado_grupos.push(GrupoDetectado {
                tipo_duplicado: "TOTP",
                etiqueta_tipo: "Mismo secreto 2FA (TOTP)",
                clave_visual: clave_vis,
                entradas: lista,
                es_app_android: es_app,
            });
        }
    }

    // 4. Detección de duplicados de la MISMA cuenta (mismo servicio y usuario normalizado)
    let restantes_cuenta: Vec<&Entrada> = candidatas_generales
        .iter()
        .copied()
        .filter(|e| !ids_procesados.contains(e.id.as_str()) && !e.usuario.trim().is_empty())
        .collect();

    let mut grupos_cuenta: HashMap<String, Vec<&Entrada>> = HashMap::new();
    for e in &restantes_cuenta {
        let serv = normalizar_servicio_duplicados(e);
        let usr_norm = normalizar_usuario_duplicados(&e.usuario, &serv);
        let clave = format!("{serv}|||{usr_norm}");
        grupos_cuenta.entry(clave).or_default().push(e);
    }

    for (_clave, lista) in grupos_cuenta {
        if lista.len() > 1 {
            let primera_clave = &lista[0].contrasena;
            let todas_misma_clave = lista.iter().all(|m| m.contrasena == *primera_clave);
            let (tipo, etiq) = if todas_misma_clave {
                ("VARIANTES", "Variante de usuario / misma clave")
            } else {
                ("MISMA_CUENTA", "Misma cuenta con distinta clave")
            };
            let clave_vis = armar_clave_visual(lista[0]);
            let es_app = lista.iter().any(|e| es_app_android_entrada(e));
            for e in &lista {
                ids_procesados.insert(e.id.as_str());
            }
            resultado_grupos.push(GrupoDetectado {
                tipo_duplicado: tipo,
                etiqueta_tipo: etiq,
                clave_visual: clave_vis,
                entradas: lista,
                es_app_android: es_app,
            });
        }
    }

    // 5. Entradas sin usuario pero con misma contraseña y servicio (copias anónimas exactas)
    let restantes_sin_user: Vec<&Entrada> = candidatas_generales
        .iter()
        .copied()
        .filter(|e| {
            !ids_procesados.contains(e.id.as_str())
                && e.usuario.trim().is_empty()
                && !e.contrasena.trim().is_empty()
        })
        .collect();

    let mut grupos_sin_user: HashMap<String, Vec<&Entrada>> = HashMap::new();
    for e in &restantes_sin_user {
        let serv = normalizar_servicio_duplicados(e);
        let pwd = &e.contrasena;
        let totp = e.secreto_totp.as_deref().unwrap_or("").replace(' ', "").to_uppercase();
        let pk = e.passkey.as_ref().map(|p| p.cred_id.trim()).unwrap_or("");
        let clave = format!("{serv}|||__sin_usuario__|||{pwd}|||{totp}|||{pk}");
        grupos_sin_user.entry(clave).or_default().push(e);
    }

    for (_clave, lista) in grupos_sin_user {
        if lista.len() > 1 {
            let clave_vis = armar_clave_visual(lista[0]);
            let es_app = lista.iter().any(|e| es_app_android_entrada(e));
            for e in &lista {
                ids_procesados.insert(e.id.as_str());
            }
            resultado_grupos.push(GrupoDetectado {
                tipo_duplicado: "IDENTICOS",
                etiqueta_tipo: "Copia anónima exacta",
                clave_visual: clave_vis,
                entradas: lista,
                es_app_android: es_app,
            });
        }
    }

    // Filtrar según el filtro activo y mapear a DuplicateGroupItem
    let mut resultado = Vec::new();

    for g in resultado_grupos {
        let _todas_misma_clave = g.entradas.iter().all(|m| m.contrasena == g.entradas[0].contrasena);
        let tiene_passkey = g.entradas.iter().any(|m| m.passkey.is_some() || !m.passkeys_secundarias.is_empty());
        let tiene_totp = g.entradas.iter().any(|m| {
            !m.secreto_totp.as_deref().unwrap_or("").trim().is_empty() || !m.totps_secundarios.is_empty()
        });
        let tiene_app = g.es_app_android;
        let tiene_web = !g.es_app_android;

        let pasa_filtro = match filtro {
            FiltroDuplicados::Todos => true,
            FiltroDuplicados::Identicos => g.tipo_duplicado == "IDENTICOS",
            FiltroDuplicados::Passkey => tiene_passkey || g.tipo_duplicado == "PASSKEY",
            FiltroDuplicados::Totp => tiene_totp || g.tipo_duplicado == "TOTP",
            FiltroDuplicados::AppsAndroid => tiene_app,
            FiltroDuplicados::SitiosWeb => tiene_web,
            FiltroDuplicados::MismaCuenta => g.tipo_duplicado == "MISMA_CUENTA",
            FiltroDuplicados::Variantes => g.tipo_duplicado == "VARIANTES",
        };

        if !pasa_filtro {
            continue;
        }

        let primer_url = g.entradas[0].urls.first().map(|s| s.as_str()).unwrap_or("");
        let titulo_servicio = extraer_nombre_servicio_canonico(primer_url, &g.entradas[0].titulo);
        let usuario_comun = if !g.clave_visual.trim().is_empty() {
            g.clave_visual
        } else if g.entradas[0].usuario.trim().is_empty() {
            "Sin usuario".to_string()
        } else {
            g.entradas[0].usuario.clone()
        };

        let ids_vec: Vec<String> = g.entradas.iter().map(|m| m.id.clone()).collect();
        let resumen_lineas: Vec<String> = g.entradas
            .iter()
            .map(|m| {
                let u = m.urls.first().map(|s| s.as_str()).unwrap_or("Sin URL");
                format!("• {} ({}) — {}", m.titulo, m.usuario, u)
            })
            .collect();

        resultado.push(DuplicateGroupItem {
            clave_grupo: SharedString::from(format!("{}:{}", titulo_servicio, usuario_comun)),
            titulo_servicio: SharedString::from(titulo_servicio),
            usuario_comun: SharedString::from(usuario_comun),
            tipo_duplicado: SharedString::from(g.tipo_duplicado),
            etiqueta_tipo: SharedString::from(g.etiqueta_tipo),
            conteo_entradas: g.entradas.len() as i32,
            ids_csv: SharedString::from(ids_vec.join(",")),
            resumen_entradas: SharedString::from(resumen_lineas.join("\n")),
        });
    }

    resultado
}

fn armar_clave_visual(entrada: &Entrada) -> String {
    let es_app = es_app_android_entrada(entrada);
    let url = entrada.urls.first().map(|s| s.as_str()).unwrap_or("");
    let nombre_servicio = extraer_nombre_servicio_canonico(url, &entrada.titulo);
    let usuario = if entrada.usuario.trim().is_empty() {
        "Sin usuario"
    } else {
        entrada.usuario.trim()
    };
    if es_app {
        format!("{nombre_servicio} (App Android) · {usuario}")
    } else {
        format!("{nombre_servicio} · {usuario}")
    }
}
