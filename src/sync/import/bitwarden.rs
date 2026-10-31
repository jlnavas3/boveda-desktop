/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Importador de respaldos en formato JSON de Bitwarden y Vaultwarden.

use crate::crypto::BvdaError;
use crate::models::{
    CampoPersonalizado, Categoria, ContenidoBoveda, DatosDocumentoIdentidad,
    DatosTarjetaBancaria, Entrada, TipoCampo, TipoEntrada,
};
use chrono::Utc;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct BwExport {
    #[serde(default)]
    folders: Vec<BwFolder>,
    #[serde(default)]
    items: Vec<BwItem>,
}

#[derive(Debug, Deserialize)]
struct BwFolder {
    id: Option<String>,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BwItem {
    id: Option<String>,
    #[serde(rename = "folderId")]
    folder_id: Option<String>,
    #[serde(rename = "type")]
    item_type: Option<i32>,
    name: Option<String>,
    notes: Option<String>,
    favorite: Option<bool>,
    login: Option<BwLogin>,
    card: Option<BwCard>,
    identity: Option<BwIdentity>,
    #[serde(default)]
    fields: Vec<BwField>,
}

#[derive(Debug, Deserialize)]
struct BwLogin {
    #[serde(default)]
    uris: Vec<BwUri>,
    username: Option<String>,
    password: Option<String>,
    totp: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BwUri {
    uri: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BwCard {
    #[serde(rename = "cardholderName")]
    cardholder_name: Option<String>,
    brand: Option<String>,
    number: Option<String>,
    #[serde(rename = "expMonth")]
    exp_month: Option<String>,
    #[serde(rename = "expYear")]
    exp_year: Option<String>,
    code: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BwIdentity {
    title: Option<String>,
    #[serde(rename = "firstName")]
    first_name: Option<String>,
    #[serde(rename = "lastName")]
    last_name: Option<String>,
    #[serde(rename = "ssn")]
    numero_identidad: Option<String>,
    #[serde(rename = "passportNumber")]
    pasaporte: Option<String>,
    #[serde(rename = "licenseNumber")]
    licencia: Option<String>,
    country: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BwField {
    name: Option<String>,
    value: Option<String>,
    #[serde(rename = "type")]
    field_type: Option<i32>,
}

/// Parsea un archivo de respaldo JSON de Bitwarden y lo transforma en una estructura `ContenidoBoveda`.
pub fn importar_bitwarden_json(json_bytes: &[u8]) -> Result<ContenidoBoveda, BvdaError> {
    let export: BwExport = serde_json::from_slice(json_bytes)?;
    let mut boveda = ContenidoBoveda::default();
    let ahora = Utc::now().timestamp_millis();

    for f in export.folders {
        if let (Some(id), Some(nombre)) = (f.id, f.name) {
            boveda.categorias.push(Categoria {
                id,
                nombre,
                icono: "carpeta".into(),
                color_hex: None,
                color: None,
                creada_en: ahora,
                modificada_en: ahora,
            });
        }
    }

    for item in export.items {
        let mut entrada = Entrada {
            id: item.id.unwrap_or_else(|| format!("bw-{}", boveda.entradas.len() + 1)),
            titulo: item.name.unwrap_or_default(),
            notas: item.notes.unwrap_or_default(),
            favorito: item.favorite.unwrap_or(false),
            creada_en: ahora,
            modificada_en: ahora,
            categorias: item.folder_id.into_iter().collect(),
            ..Default::default()
        };

        // Mapeo de campos personalizados
        for f in item.fields {
            if let Some(etiqueta) = f.name {
                let valor = f.value.unwrap_or_default();
                let es_oculto = f.field_type == Some(1);
                entrada.campos_personalizados.push(CampoPersonalizado {
                    id: format!("cp-{}", entrada.campos_personalizados.len() + 1),
                    etiqueta,
                    valor,
                    tipo: if es_oculto { TipoCampo::Oculto } else { TipoCampo::Texto },
                    es_sensible: es_oculto,
                    es_obligatorio: false,
                    formato: None,
                });
            }
        }

        match item.item_type {
            Some(1) => {
                // 1 = Login
                entrada.tipo = TipoEntrada::Login;
                if let Some(login) = item.login {
                    entrada.usuario = login.username.unwrap_or_default();
                    entrada.contrasena = login.password.unwrap_or_default();
                    for u in login.uris {
                        if let Some(uri) = u.uri {
                            if !uri.trim().is_empty() {
                                entrada.urls.push(uri);
                            }
                        }
                    }
                    if let Some(totp_raw) = login.totp {
                        entrada.secreto_totp = extraer_secreto_totp(&totp_raw);
                    }
                }
            }
            Some(2) => {
                // 2 = Secure Note
                entrada.tipo = TipoEntrada::Nota;
            }
            Some(3) => {
                // 3 = Card
                entrada.tipo = TipoEntrada::Tarjeta;
                if let Some(card) = item.card {
                    let vencimiento = match (card.exp_month, card.exp_year) {
                        (Some(m), Some(y)) if !m.is_empty() && !y.is_empty() => {
                            let y_trunc = if y.len() > 2 { &y[y.len() - 2..] } else { &y };
                            format!("{:0>2}/{}", m, y_trunc)
                        }
                        _ => String::new(),
                    };

                    entrada.datos_tarjeta = Some(DatosTarjetaBancaria {
                        titular: card.cardholder_name.unwrap_or_default(),
                        numero: card.number.unwrap_or_default(),
                        vencimiento,
                        cvv: card.code.unwrap_or_default(),
                        pin: String::new(),
                        tipo: "Crédito".into(),
                        marca: card.brand.unwrap_or_default(),
                    });
                }
            }
            Some(4) => {
                // 4 = Identity
                entrada.tipo = TipoEntrada::Identidad;
                if let Some(ident) = item.identity {
                    let mut nombre_parts = Vec::new();
                    if let Some(t) = ident.title { if !t.is_empty() { nombre_parts.push(t); } }
                    if let Some(f) = ident.first_name { if !f.is_empty() { nombre_parts.push(f); } }
                    if let Some(l) = ident.last_name { if !l.is_empty() { nombre_parts.push(l); } }

                    let numero = ident.numero_identidad
                        .or(ident.pasaporte)
                        .or(ident.licencia)
                        .unwrap_or_default();

                    entrada.datos_identidad = Some(DatosDocumentoIdentidad {
                        tipo_documento: "DNI".into(),
                        numero,
                        nombre_completo: nombre_parts.join(" "),
                        expedicion: String::new(),
                        caducidad: String::new(),
                        pais_emisor: ident.country.unwrap_or_default(),
                    });
                }
            }
            _ => {
                entrada.tipo = TipoEntrada::Login;
            }
        }

        boveda.entradas.push(entrada);
    }

    Ok(boveda)
}

fn extraer_secreto_totp(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }

    if let Some(pos) = raw.find("secret=") {
        let sub = &raw[pos + 7..];
        let fin = sub.find('&').unwrap_or(sub.len());
        let sec = &sub[..fin];
        if !sec.is_empty() {
            return Some(sec.to_string());
        }
    }

    Some(raw.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_importar_bitwarden_json() {
        let json = r#"{
            "folders": [
                { "id": "f-1", "name": "Servicios" }
            ],
            "items": [
                {
                    "id": "item-1",
                    "folderId": "f-1",
                    "type": 1,
                    "name": "Google",
                    "login": {
                        "username": "user@gmail.com",
                        "password": "pass123",
                        "uris": [ { "uri": "https://accounts.google.com" } ],
                        "totp": "otpauth://totp/Google:user?secret=JBSWY3DPEHPK3PXP"
                    }
                },
                {
                    "id": "item-2",
                    "type": 3,
                    "name": "Visa Banco",
                    "card": {
                        "cardholderName": "Juan Perez",
                        "number": "4532111122223333",
                        "expMonth": "05",
                        "expYear": "2029",
                        "code": "456",
                        "brand": "Visa"
                    }
                }
            ]
        }"#;

        let boveda = importar_bitwarden_json(json.as_bytes()).expect("importar json");
        assert_eq!(boveda.categorias.len(), 1);
        assert_eq!(boveda.categorias[0].nombre, "Servicios");
        assert_eq!(boveda.entradas.len(), 2);

        let e1 = &boveda.entradas[0];
        assert_eq!(e1.titulo, "Google");
        assert_eq!(e1.usuario, "user@gmail.com");
        assert_eq!(e1.secreto_totp.as_deref(), Some("JBSWY3DPEHPK3PXP"));

        let e2 = &boveda.entradas[1];
        assert_eq!(e2.tipo, TipoEntrada::Tarjeta);
        let tarjeta = e2.datos_tarjeta.as_ref().unwrap();
        assert_eq!(tarjeta.titular, "Juan Perez");
        assert_eq!(tarjeta.vencimiento, "05/29");
    }
}
