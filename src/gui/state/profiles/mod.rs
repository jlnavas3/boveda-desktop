/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Módulo de gestión CRUD de Perfiles / Identidades.

pub mod delete_profile;
pub mod list_profiles;
pub mod save_profile;

pub use delete_profile::eliminar_perfil_boveda;
pub use list_profiles::listar_perfiles_items;
pub use save_profile::guardar_o_crear_perfil;
