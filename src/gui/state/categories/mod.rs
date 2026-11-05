/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Módulo de gestión CRUD de categorías de la bóveda.

pub mod delete_category;
pub mod save_category;

pub use delete_category::eliminar_categoria_boveda;
pub use save_category::guardar_o_crear_categoria;
