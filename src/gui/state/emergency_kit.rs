/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Generador del documento de Kit de Emergencia de Rescate Soberano.

pub fn generar_kit_emergencia_texto(
    ruta_boveda: &str,
    perfil_argon2: &str,
    conteo_entradas: usize,
    conteo_categorias: usize,
) -> String {
    let fecha = chrono::Local::now().format("%d/%m/%Y %H:%M").to_string();
    format!(
        "====================================================================\n\
         KIT DE EMERGENCIA SOBERANO — BÓVEDA LOCAL DESKTOP\n\
         ====================================================================\n\
         Fecha de emisión:          {fecha}\n\
         Archivo de bóveda (.bvda): {ruta_boveda}\n\
         Algoritmo de cifrado:      AES-256-GCM + Compresión BLZ1\n\
         Derivación KDF activa:     Argon2id ({perfil_argon2})\n\
         Credenciales activas:      {conteo_entradas}\n\
         Categorías registradas:    {conteo_categorias}\n\
         --------------------------------------------------------------------\n\
         ESPACIO PARA ANOTACIÓN FÍSICA DE LA CONTRASEÑA MAESTRA\n\
         (Escriba a mano tras imprimir y guarde en caja fuerte física)\n\n\
         Contraseña Maestra:\n\
         ____________________________________________________________________\n\n\
         ____________________________________________________________________\n\n\
         Ubicación de respaldos (.bvda):\n\
         1. _________________________________________________________________\n\
         2. _________________________________________________________________\n\
         ====================================================================\n"
    )
}
