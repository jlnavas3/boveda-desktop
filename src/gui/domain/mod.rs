/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Submódulo desacoplado para resolución canónica de dominios web, marcas y URLs.

pub mod extract_host;
pub mod official_brands;
pub mod domain_root;
pub mod service_extractor;

pub use extract_host::extraer_host_limpio;
pub use official_brands::obtener_marca_oficial;
pub use domain_root::{extraer_raiz_y_marca, SUFIJOS_PUBLICOS_COMPUESTOS};
pub use service_extractor::extraer_nombre_servicio_canonico;
