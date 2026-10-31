/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Modelos inmutables de datos coincidentes al 100% con Bóveda Local Android.

pub mod adjunto;
pub mod cambio_contrasena;
pub mod campo_personalizado;
pub mod categoria;
pub mod contacto;
pub mod contenido;
pub mod cuenta_bancaria;
pub mod entrada;
pub mod identidad_doc;
pub mod passkey;
pub mod perfil;
pub mod servidor;
pub mod tarjeta;
pub mod tipo_campo;
pub mod tipo_entrada;
pub mod totp_secundario;
pub mod vault_config;
pub mod wallet;
pub mod wifi;

pub use adjunto::AdjuntoBoveda;
pub use cambio_contrasena::CambioContrasena;
pub use campo_personalizado::CampoPersonalizado;
pub use categoria::Categoria;
pub use contacto::DatosContacto;
pub use contenido::ContenidoBoveda;
pub use cuenta_bancaria::DatosCuentaBancaria;
pub use entrada::Entrada;
pub use identidad_doc::DatosDocumentoIdentidad;
pub use passkey::DatosPasskey;
pub use perfil::Perfil;
pub use servidor::DatosServidorSsh;
pub use tarjeta::DatosTarjetaBancaria;
pub use tipo_campo::TipoCampo;
pub use tipo_entrada::TipoEntrada;
pub use totp_secundario::TotpSecundario;
pub use vault_config::ConfiguracionBovedaExportable;
pub use wallet::DatosCriptoWallet;
pub use wifi::DatosRedWifi;
