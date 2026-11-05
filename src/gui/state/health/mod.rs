/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Módulo de auditoría y acciones correctivas de Salud de la Bóveda.

pub mod analyze_vault_health;
pub mod common_passwords_check;
pub mod consolidated_health_counter;
pub mod health_tab_kind;
pub mod quick_rotate_password;

pub use analyze_vault_health::analizar_salud_boveda;
pub use common_passwords_check::es_contrasena_comun;
pub use consolidated_health_counter::contar_problemas_salud_boveda_canonico;
pub use health_tab_kind::PestanaSalud;
pub use quick_rotate_password::rotar_contrasena_rapida;
