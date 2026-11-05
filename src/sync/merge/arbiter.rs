/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionArbitro {
    SinCambios,
    FastForwardLocal,   // Gana versión local sin tocar campos
    FastForwardRemoto,  // Gana versión remota sin tocar campos
    SplitBrain,         // Bifurcación: invocar fusionar_entradas() y sellar con max(revA, revB) + 1
}

/// Evalúa dos versiones de la misma entrada (mismo UUID) de forma 100% determinista.
/// Espejo matemático exacto de ArbitroSincronizacion.kt en Android.
#[allow(clippy::ref_option)]
pub fn evaluar_estado_entrada(
    hash_local: &[u8; 32],
    last_sync_local: &Option<[u8; 32]>,
    rev_local: u64,
    hash_remoto: &[u8; 32],
    last_sync_remoto: &Option<[u8; 32]>,
    rev_remoto: u64,
) -> DecisionArbitro {
    // 1. Idénticas en contenido: nada que hacer
    if hash_local == hash_remoto {
        return DecisionArbitro::SinCambios;
    }

    // 2. Fast-Forward Local: Desktop avanzó, pero Android sigue exactamente
    //    en el estado del último sync que recuerda Desktop.
    if last_sync_local.as_ref() == Some(hash_remoto) && rev_local > rev_remoto {
        return DecisionArbitro::FastForwardLocal;
    }

    // 3. Fast-Forward Remoto: Android avanzó, pero Desktop sigue exactamente
    //    en el estado del último sync que recuerda Android.
    if last_sync_remoto.as_ref() == Some(hash_local) && rev_remoto > rev_local {
        return DecisionArbitro::FastForwardRemoto;
    }

    // 4. Fork simétrico (rev: 5 vs rev: 5), fork asimétrico (rev: 6 vs rev: 5 con base común rev: 4),
    //    o primer sync sin ancestro (None): ¡Split-Brain Concurrente!
    DecisionArbitro::SplitBrain
}
