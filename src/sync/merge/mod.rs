/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Submódulo de fusión aditiva y resolución de colisiones sin pérdida de datos.

pub mod entry_merge;
pub mod passkey_dedup;
pub mod submodel_merge;
pub mod vault_merge;

pub use entry_merge::fusionar_entradas;
pub use passkey_dedup::{consolidar_passkey, son_misma_passkey};
pub use submodel_merge::{fusionar_submodelos, ResultadoFusionNativa};
pub use vault_merge::fusionar_bovedas;
