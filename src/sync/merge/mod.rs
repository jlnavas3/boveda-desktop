/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Submódulo de fusión aditiva y resolución de colisiones sin pérdida de datos.

pub mod arbiter;
pub mod diff_mask;
pub mod entry_hasher;
pub mod entry_merge;
pub mod passkey_dedup;
pub mod submodel_merge;
pub mod sync_report;
pub mod vault_merge;

pub use arbiter::{evaluar_estado_entrada, DecisionArbitro};
pub use diff_mask::{calcular_diff, de_entrada_nueva, es_url_web, extraer_paquete_app, MascaraCampos};
pub use entry_hasher::calcular_content_hash;
pub use entry_merge::fusionar_entradas;
pub use passkey_dedup::{consolidar_passkey, son_misma_passkey};
pub use submodel_merge::{fusionar_submodelos, ResultadoFusionNativa};
pub use sync_report::{InformeSincronizacion, ItemCambioSync, TipoCambioSync};
pub use vault_merge::{fusionar_bovedas, fusionar_bovedas_con_informe};
