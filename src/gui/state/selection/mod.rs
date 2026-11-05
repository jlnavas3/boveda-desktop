/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Submódulos desacoplados para el modo de selección simple y múltiple (estilo Android `ControladorSeleccionLista`).

pub mod batch_assign_categories;
pub mod batch_rename;
pub mod batch_toggle_favorite;
pub mod batch_toggle_health_ignore;
pub mod batch_toggle_pinned;
pub mod batch_trash_actions;
pub mod entry_unifier;
pub mod selection_state;
pub mod unification_analyzer;

pub use batch_assign_categories::asignar_categorias_lote;
pub use batch_rename::renombrar_lote;
pub use batch_toggle_favorite::alternar_favorito_lote;
pub use batch_toggle_health_ignore::alternar_ignorar_salud_lote;
pub use batch_toggle_pinned::alternar_anclado_lote;
pub use batch_trash_actions::procesar_papelera_lote;
pub use entry_unifier::unificar_entradas_seleccionadas;
pub use selection_state::EstadoSeleccionLista;
pub use unification_analyzer::analizar_discrepancias_unificacion;
