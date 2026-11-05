/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

use boveda_core::models::Entrada;
use boveda_core::sync::{calcular_content_hash, evaluar_estado_entrada, DecisionArbitro};

#[test]
fn test_sin_cambios() {
    let hash_comun = [0x42u8; 32];
    let last_sync = Some([0x11u8; 32]);

    let decision = evaluar_estado_entrada(
        &hash_comun,
        &last_sync,
        5,
        &hash_comun,
        &last_sync,
        5,
    );

    assert_eq!(decision, DecisionArbitro::SinCambios);
}

#[test]
fn test_fast_forward_local() {
    // Desktop avanzó a rev 5 con nuevo contenido (hash_local).
    // Su last_sync recuerda el hash_base (Android en rev 4).
    let hash_base = [0x01u8; 32];
    let hash_local = [0x02u8; 32];
    let last_sync_local = Some(hash_base);
    let last_sync_remoto = Some(hash_base);

    let decision = evaluar_estado_entrada(
        &hash_local,
        &last_sync_local,
        5,
        &hash_base, // Android sigue en el hash base
        &last_sync_remoto,
        4,
    );

    assert_eq!(decision, DecisionArbitro::FastForwardLocal);
}

#[test]
fn test_fast_forward_remoto() {
    // Android avanzó a rev 5 con nuevo contenido (hash_remoto).
    // Su last_sync recuerda el hash_base (Desktop en rev 4).
    let hash_base = [0x01u8; 32];
    let hash_remoto = [0x03u8; 32];
    let last_sync_local = Some(hash_base);
    let last_sync_remoto = Some(hash_base);

    let decision = evaluar_estado_entrada(
        &hash_base, // Desktop sigue en el hash base
        &last_sync_local,
        4,
        &hash_remoto,
        &last_sync_remoto,
        5,
    );

    assert_eq!(decision, DecisionArbitro::FastForwardRemoto);
}

#[test]
fn test_split_brain_simetrico() {
    // rev 5 vs rev 5 con hashes distintos
    let hash_local = [0xAAu8; 32];
    let hash_remoto = [0xBBu8; 32];
    let base_comun = Some([0x00u8; 32]);

    let decision = evaluar_estado_entrada(
        &hash_local,
        &base_comun,
        5,
        &hash_remoto,
        &base_comun,
        5,
    );

    assert_eq!(decision, DecisionArbitro::SplitBrain);
}

#[test]
fn test_split_brain_asimetrico() {
    // rev 6 vs rev 5 ambos partiendo de rev 4 (base_comun).
    // Ninguno tiene el hash del otro como last_sync.
    let base_comun = [0x10u8; 32];
    let hash_local = [0x20u8; 32];
    let hash_remoto = [0x30u8; 32];

    let decision = evaluar_estado_entrada(
        &hash_local,
        &Some(base_comun),
        6,
        &hash_remoto,
        &Some(base_comun),
        5,
    );

    assert_eq!(decision, DecisionArbitro::SplitBrain);
}

#[test]
fn test_primer_sync_sin_ancestro() {
    // None vs None con hashes distintos: Split-Brain Concurrente
    let hash_local = [0x12u8; 32];
    let hash_remoto = [0x34u8; 32];

    let decision = evaluar_estado_entrada(
        &hash_local,
        &None,
        1,
        &hash_remoto,
        &None,
        1,
    );

    assert_eq!(decision, DecisionArbitro::SplitBrain);
}

#[test]
fn test_hasher_determinista() {
    let mut e1 = Entrada {
        id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        titulo: "GitHub".to_string(),
        usuario: "octocat".to_string(),
        contrasena: "CorrectHorseBatteryStaple".to_string(),
        notas: "Cuenta corporativa".to_string(),
        secreto_totp: Some("JBSWY3DPEHPK3PXP".to_string()),
        urls: vec!["https://github.com".to_string(), "https://gist.github.com".to_string()],
        revision: 1,
        last_sync_hash: None,
        ..Default::default()
    };

    let hash_original = calcular_content_hash(&e1);

    // Cambiar solo la revisión o el last_sync_hash NO altera el content_hash
    e1.revision = 42;
    e1.last_sync_hash = Some([0x99u8; 32]);
    let hash_post_metadata = calcular_content_hash(&e1);
    assert_eq!(hash_original, hash_post_metadata);

    // Cambiar un campo nuclear sí altera el content_hash
    e1.contrasena = "NuevaPassword123!".to_string();
    let hash_modificado = calcular_content_hash(&e1);
    assert_ne!(hash_original, hash_modificado);
}

#[test]
fn test_entrada_serde_retrocompatibilidad_y_hex() {
    // 1. Deserializar JSON sin 'revision' ni 'lastSyncHash' (bóvedas anteriores)
    let json_antiguo = r#"{
        "id": "e-1",
        "titulo": "Google",
        "usuario": "user@gmail.com",
        "contrasena": "secret"
    }"#;

    let entrada_des: Entrada = serde_json::from_str(json_antiguo).unwrap();
    assert_eq!(entrada_des.revision, 1);
    assert_eq!(entrada_des.last_sync_hash, None);

    // 2. Serializar y deserializar con last_sync_hash hex de 64 caracteres
    let mut entrada = entrada_des;
    entrada.revision = 3;
    let hash_ejemplo = [0xABu8; 32];
    entrada.last_sync_hash = Some(hash_ejemplo);

    let json_nuevo = serde_json::to_string(&entrada).unwrap();
    assert!(json_nuevo.contains(r#""revision":3"#));
    assert!(json_nuevo.contains(r#""lastSyncHash":"abababababababababababababababababababababababababababababababab""#));

    let roundtrip: Entrada = serde_json::from_str(&json_nuevo).unwrap();
    assert_eq!(roundtrip.revision, 3);
    assert_eq!(roundtrip.last_sync_hash, Some(hash_ejemplo));
}
