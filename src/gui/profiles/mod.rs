/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Submódulo desacoplado para gestión de perfiles e identidades organizativas.

pub mod smart_resolver;

pub use smart_resolver::{es_mismo_documento, resolver_perfil_para_entrada};
