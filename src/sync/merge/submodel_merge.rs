/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Fusión no destructiva de submodelos nativos especializados.
//! Réplica fiel de FusionadorSubmodelosNativos.kt de Bóveda Local Android.

use crate::models::{
    DatosContacto, DatosCuentaBancaria, DatosCriptoWallet, DatosDocumentoIdentidad,
    DatosRedWifi, DatosServidorSsh, DatosTarjetaBancaria, Entrada, TipoEntrada,
};

#[derive(Debug, Clone, Default)]
pub struct ResultadoFusionNativa {
    pub datos_tarjeta: Option<DatosTarjetaBancaria>,
    pub datos_wifi: Option<DatosRedWifi>,
    pub datos_cuenta_bancaria: Option<DatosCuentaBancaria>,
    pub datos_identidad: Option<DatosDocumentoIdentidad>,
    pub datos_servidor: Option<DatosServidorSsh>,
    pub datos_wallet: Option<DatosCriptoWallet>,
    pub datos_contacto: Option<DatosContacto>,
    pub notas_de_respaldo: Vec<String>,
}

pub fn fusionar_submodelos(principal: &Entrada, secundarias: &[Entrada]) -> ResultadoFusionNativa {
    let mut notas_respaldo = Vec::new();

    let mut res_tarjeta = principal.datos_tarjeta.clone();
    let mut res_wifi = principal.datos_wifi.clone();
    let mut res_cuenta = principal.datos_cuenta_bancaria.clone();
    let mut res_identidad = principal.datos_identidad.clone();
    let mut res_servidor = principal.datos_servidor.clone();
    let mut res_wallet = principal.datos_wallet.clone();
    let mut res_contacto = principal.datos_contacto.clone();

    for sec in secundarias {
        match principal.tipo {
            TipoEntrada::Tarjeta => {
                if let Some(sec_t) = &sec.datos_tarjeta {
                    let base = res_tarjeta.take().unwrap_or_default();
                    let num_base_digits: String = base.numero.chars().filter(|c| c.is_ascii_digit()).collect();
                    let num_sec_digits: String = sec_t.numero.chars().filter(|c| c.is_ascii_digit()).collect();

                    if !num_base_digits.is_empty() && !num_sec_digits.is_empty() && num_base_digits != num_sec_digits {
                        notas_respaldo.push(format!(
                            "[Tarjeta secundaria respaldada al unificar: Titular: {} | Número: {} | Vencimiento: {} | CVV: {} | PIN: {}]",
                            if sec_t.titular.is_empty() { "N/A" } else { &sec_t.titular },
                            sec_t.numero,
                            if sec_t.vencimiento.is_empty() { "N/A" } else { &sec_t.vencimiento },
                            if sec_t.cvv.is_empty() { "N/A" } else { &sec_t.cvv },
                            if sec_t.pin.is_empty() { "N/A" } else { &sec_t.pin },
                        ));
                        res_tarjeta = Some(base);
                    } else {
                        res_tarjeta = Some(DatosTarjetaBancaria {
                            titular: if !base.titular.is_empty() { base.titular } else { sec_t.titular.clone() },
                            numero: if !base.numero.is_empty() { base.numero } else { sec_t.numero.clone() },
                            vencimiento: if !base.vencimiento.is_empty() { base.vencimiento } else { sec_t.vencimiento.clone() },
                            cvv: if !base.cvv.is_empty() { base.cvv } else { sec_t.cvv.clone() },
                            pin: if !base.pin.is_empty() { base.pin } else { sec_t.pin.clone() },
                            marca: if !base.marca.is_empty() { base.marca } else { sec_t.marca.clone() },
                            tipo: if !base.tipo.is_empty() && base.tipo != "Crédito" { base.tipo } else { sec_t.tipo.clone() },
                        });
                    }
                }
            }
            TipoEntrada::Wifi => {
                if let Some(sec_w) = &sec.datos_wifi {
                    let base = res_wifi.take().unwrap_or_default();
                    let clave_base = if !base.contrasena.is_empty() { base.contrasena.clone() } else { principal.contrasena.clone() };
                    let clave_sec = if !sec_w.contrasena.is_empty() { sec_w.contrasena.clone() } else { sec.contrasena.clone() };

                    if !clave_base.is_empty() && !clave_sec.is_empty() && clave_base != clave_sec {
                        notas_respaldo.push(format!(
                            "[Contraseña Wi-Fi secundaria respaldada de red '{}': {}]",
                            if !sec_w.ssid.is_empty() { &sec_w.ssid } else { &principal.titulo },
                            clave_sec
                        ));
                    }

                    res_wifi = Some(DatosRedWifi {
                        ssid: if !base.ssid.is_empty() { base.ssid } else { sec_w.ssid.clone() },
                        contrasena: if !clave_base.is_empty() { clave_base } else { clave_sec },
                        tipo_seguridad: if !base.tipo_seguridad.is_empty() { base.tipo_seguridad } else { sec_w.tipo_seguridad.clone() },
                        oculta: base.oculta || sec_w.oculta,
                    });
                }
            }
            TipoEntrada::CuentaBancaria => {
                if let Some(sec_c) = &sec.datos_cuenta_bancaria {
                    let base = res_cuenta.take().unwrap_or_default();
                    let iban_base: String = base.cbu_clabe.chars().chain(base.numero_cuenta.chars()).filter(|c| c.is_alphanumeric()).collect();
                    let iban_sec: String = sec_c.cbu_clabe.chars().chain(sec_c.numero_cuenta.chars()).filter(|c| c.is_alphanumeric()).collect();

                    if !iban_base.is_empty() && !iban_sec.is_empty() && iban_base != iban_sec {
                        notas_respaldo.push(format!(
                            "[Cuenta bancaria secundaria respaldada al unificar: Banco: {} | Titular: {} | Número: {} | IBAN/CBU: {} | SWIFT: {}]",
                            if sec_c.banco.is_empty() { "N/A" } else { &sec_c.banco },
                            if sec_c.titular.is_empty() { "N/A" } else { &sec_c.titular },
                            if sec_c.numero_cuenta.is_empty() { "N/A" } else { &sec_c.numero_cuenta },
                            if sec_c.cbu_clabe.is_empty() { "N/A" } else { &sec_c.cbu_clabe },
                            if sec_c.swift_bic.is_empty() { "N/A" } else { &sec_c.swift_bic },
                        ));
                        res_cuenta = Some(base);
                    } else {
                        res_cuenta = Some(DatosCuentaBancaria {
                            banco: if !base.banco.is_empty() { base.banco } else { sec_c.banco.clone() },
                            titular: if !base.titular.is_empty() { base.titular } else { sec_c.titular.clone() },
                            tipo_cuenta: if !base.tipo_cuenta.is_empty() && base.tipo_cuenta != "Corriente" { base.tipo_cuenta } else { sec_c.tipo_cuenta.clone() },
                            numero_cuenta: if !base.numero_cuenta.is_empty() { base.numero_cuenta } else { sec_c.numero_cuenta.clone() },
                            swift_bic: if !base.swift_bic.is_empty() { base.swift_bic } else { sec_c.swift_bic.clone() },
                            cbu_clabe: if !base.cbu_clabe.is_empty() { base.cbu_clabe } else { sec_c.cbu_clabe.clone() },
                        });
                    }
                }
            }
            TipoEntrada::Identidad => {
                if let Some(sec_i) = &sec.datos_identidad {
                    let base = res_identidad.take().unwrap_or_default();
                    if !base.numero.is_empty() && !sec_i.numero.is_empty() && base.numero != sec_i.numero {
                        notas_respaldo.push(format!(
                            "[Documento de identidad secundario respaldado: Tipo: {} | Número: {} | Nombre: {} | País: {}]",
                            sec_i.tipo_documento, sec_i.numero, sec_i.nombre_completo, sec_i.pais_emisor
                        ));
                        res_identidad = Some(base);
                    } else {
                        res_identidad = Some(DatosDocumentoIdentidad {
                            tipo_documento: if !base.tipo_documento.is_empty() && base.tipo_documento != "CI" { base.tipo_documento } else { sec_i.tipo_documento.clone() },
                            numero: if !base.numero.is_empty() { base.numero } else { sec_i.numero.clone() },
                            nombre_completo: if !base.nombre_completo.is_empty() { base.nombre_completo } else { sec_i.nombre_completo.clone() },
                            expedicion: if !base.expedicion.is_empty() { base.expedicion } else { sec_i.expedicion.clone() },
                            caducidad: if !base.caducidad.is_empty() { base.caducidad } else { sec_i.caducidad.clone() },
                            pais_emisor: if !base.pais_emisor.is_empty() { base.pais_emisor } else { sec_i.pais_emisor.clone() },
                        });
                    }
                }
            }
            TipoEntrada::Servidor => {
                if let Some(sec_s) = &sec.datos_servidor {
                    let base = res_servidor.take().unwrap_or_default();
                    if (!base.clave_privada.is_empty() && !sec_s.clave_privada.is_empty() && base.clave_privada != sec_s.clave_privada)
                        || (!base.host.is_empty() && !sec_s.host.is_empty() && base.host != sec_s.host)
                    {
                        notas_respaldo.push(format!(
                            "[Servidor SSH alternativo respaldado: Host: {}:{} | Usuario: {} | Clave Pública: {}]",
                            sec_s.host, sec_s.puerto, sec_s.usuario, sec_s.clave_publica
                        ));
                        res_servidor = Some(base);
                    } else {
                        res_servidor = Some(DatosServidorSsh {
                            host: if !base.host.is_empty() { base.host } else { sec_s.host.clone() },
                            puerto: if !base.puerto.is_empty() && base.puerto != "22" { base.puerto } else { sec_s.puerto.clone() },
                            usuario: if !base.usuario.is_empty() { base.usuario } else { sec_s.usuario.clone() },
                            clave_privada: if !base.clave_privada.is_empty() { base.clave_privada } else { sec_s.clave_privada.clone() },
                            clave_publica: if !base.clave_publica.is_empty() { base.clave_publica } else { sec_s.clave_publica.clone() },
                        });
                    }
                }
            }
            TipoEntrada::Wallet => {
                if let Some(sec_w) = &sec.datos_wallet {
                    let base = res_wallet.take().unwrap_or_default();
                    if !base.direccion_publica.is_empty() && !sec_w.direccion_publica.is_empty() && base.direccion_publica != sec_w.direccion_publica {
                        notas_respaldo.push(format!(
                            "[Wallet alternativa respaldada al unificar: Red: {} | Dirección: {}]",
                            sec_w.red, sec_w.direccion_publica
                        ));
                        res_wallet = Some(base);
                    } else {
                        res_wallet = Some(DatosCriptoWallet {
                            red: if !base.red.is_empty() && base.red != "Bitcoin" { base.red } else { sec_w.red.clone() },
                            direccion_publica: if !base.direccion_publica.is_empty() { base.direccion_publica } else { sec_w.direccion_publica.clone() },
                            frase_semilla: if !base.frase_semilla.is_empty() { base.frase_semilla } else { sec_w.frase_semilla.clone() },
                            clave_privada: if !base.clave_privada.is_empty() { base.clave_privada } else { sec_w.clave_privada.clone() },
                        });
                    }
                }
            }
            TipoEntrada::Contacto => {
                if let Some(sec_c) = &sec.datos_contacto {
                    let base = res_contacto.take().unwrap_or_default();
                    res_contacto = Some(DatosContacto {
                        nombre: if !base.nombre.is_empty() { base.nombre } else { sec_c.nombre.clone() },
                        telefono_secundario: if !base.telefono_secundario.is_empty() { base.telefono_secundario } else { sec_c.telefono_secundario.clone() },
                        correo: if !base.correo.is_empty() { base.correo } else { sec_c.correo.clone() },
                        direccion: if !base.direccion.is_empty() { base.direccion } else { sec_c.direccion.clone() },
                        codigo_postal: if !base.codigo_postal.is_empty() { base.codigo_postal } else { sec_c.codigo_postal.clone() },
                        ciudad: if !base.ciudad.is_empty() { base.ciudad } else { sec_c.ciudad.clone() },
                        provincia: if !base.provincia.is_empty() { base.provincia } else { sec_c.provincia.clone() },
                        pais: if !base.pais.is_empty() { base.pais } else { sec_c.pais.clone() },
                        empresa: if !base.empresa.is_empty() { base.empresa } else { sec_c.empresa.clone() },
                    });
                }
            }
            _ => {}
        }
    }

    ResultadoFusionNativa {
        datos_tarjeta: res_tarjeta,
        datos_wifi: res_wifi,
        datos_cuenta_bancaria: res_cuenta,
        datos_identidad: res_identidad,
        datos_servidor: res_servidor,
        datos_wallet: res_wallet,
        datos_contacto: res_contacto,
        notas_de_respaldo: notas_respaldo,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fusion_tarjeta_enriquecimiento() {
        let p = Entrada {
            tipo: TipoEntrada::Tarjeta,
            datos_tarjeta: Some(DatosTarjetaBancaria {
                titular: "Jose Luis".into(),
                numero: "4532".into(),
                ..Default::default()
            }),
            ..Default::default()
        };

        let s = Entrada {
            tipo: TipoEntrada::Tarjeta,
            datos_tarjeta: Some(DatosTarjetaBancaria {
                numero: "4532".into(),
                cvv: "123".into(),
                vencimiento: "12/28".into(),
                ..Default::default()
            }),
            ..Default::default()
        };

        let res = fusionar_submodelos(&p, &[s]);
        let tarjeta = res.datos_tarjeta.unwrap();
        assert_eq!(tarjeta.titular, "Jose Luis");
        assert_eq!(tarjeta.cvv, "123");
        assert_eq!(tarjeta.vencimiento, "12/28");
        assert!(res.notas_de_respaldo.is_empty());
    }

    #[test]
    fn test_fusion_tarjeta_conflicto_genera_nota() {
        let p = Entrada {
            tipo: TipoEntrada::Tarjeta,
            datos_tarjeta: Some(DatosTarjetaBancaria {
                numero: "4532000011112222".into(),
                ..Default::default()
            }),
            ..Default::default()
        };

        let s = Entrada {
            tipo: TipoEntrada::Tarjeta,
            datos_tarjeta: Some(DatosTarjetaBancaria {
                numero: "5555444433332222".into(),
                titular: "Tarjeta 2".into(),
                ..Default::default()
            }),
            ..Default::default()
        };

        let res = fusionar_submodelos(&p, &[s]);
        assert_eq!(res.notas_de_respaldo.len(), 1);
        assert!(res.notas_de_respaldo[0].contains("Tarjeta secundaria respaldada"));
    }
}
