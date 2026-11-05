/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Submódulos desacoplados para la ficha de detalle de una entrada y la gestión de credenciales alternativas.

pub mod build_entry_detail;
pub mod delete_password_history;
pub mod delete_secondary_passkey;
pub mod delete_secondary_totp;
pub mod format_blocks_4x4;
pub mod promote_secondary_passkey;
pub mod promote_secondary_totp;
pub mod restore_password_history;

pub use build_entry_detail::construir_detalle_entrada;
pub use delete_password_history::eliminar_contrasena_historial;
pub use delete_secondary_passkey::eliminar_passkey_secundaria;
pub use delete_secondary_totp::eliminar_totp_secundario;
pub use format_blocks_4x4::formatear_bloques_4x4;
pub use promote_secondary_passkey::promover_passkey_secundaria;
pub use promote_secondary_totp::promover_totp_secundario;
pub use restore_password_history::restaurar_contrasena_historial;
