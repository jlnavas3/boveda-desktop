/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

use boveda_core::models::{ContenidoBoveda, Entrada};
use boveda_core::sync::{calcular_content_hash, fusionar_bovedas};

#[test]
fn test_fast_forward_local() {
    // 1. Estado base compartido
    let mut e_base = Entrada {
        id: "e-ff-1".to_string(),
        titulo: "Amazon".to_string(),
        usuario: "comprador@mail.com".to_string(),
        contrasena: "claveBase123".to_string(),
        revision: 1,
        ..Default::default()
    };
    let hash_base = calcular_content_hash(&e_base);
    e_base.last_sync_hash = Some(hash_base);

    // Desktop avanzó a rev 2 (cambió la contraseña)
    let mut e_local = e_base.clone();
    e_local.contrasena = "claveDesktopActualizada".to_string();
    e_local.revision = 2;
    e_local.modificada_en = 2000;
    let hash_local = calcular_content_hash(&e_local);

    // Android se quedó en el estado base (rev 1)
    let e_remota = e_base;

    let boveda_local = ContenidoBoveda {
        entradas: vec![e_local],
        ..Default::default()
    };
    let boveda_remota = ContenidoBoveda {
        entradas: vec![e_remota],
        ..Default::default()
    };

    let fusionada = fusionar_bovedas(&boveda_local, &boveda_remota);

    assert_eq!(fusionada.entradas.len(), 1);
    let res = &fusionada.entradas[0];
    assert_eq!(res.contrasena, "claveDesktopActualizada");
    assert_eq!(res.revision, 2);
    assert_eq!(res.last_sync_hash, Some(hash_local));
}

#[test]
fn test_fast_forward_remoto() {
    // 1. Estado base compartido
    let mut e_base = Entrada {
        id: "e-ff-2".to_string(),
        titulo: "Netflix".to_string(),
        usuario: "streamer@mail.com".to_string(),
        contrasena: "claveBase123".to_string(),
        revision: 1,
        ..Default::default()
    };
    let hash_base = calcular_content_hash(&e_base);
    e_base.last_sync_hash = Some(hash_base);

    // Desktop se quedó en el estado base (rev 1)
    let e_local = e_base.clone();

    // Android avanzó a rev 2 (cambió notas y contraseña)
    let mut e_remota = e_base;
    e_remota.contrasena = "claveAndroidNueva".to_string();
    e_remota.notas = "Actualizado desde el móvil".to_string();
    e_remota.revision = 2;
    e_remota.modificada_en = 2500;
    let hash_remoto = calcular_content_hash(&e_remota);

    let boveda_local = ContenidoBoveda {
        entradas: vec![e_local],
        ..Default::default()
    };
    let boveda_remota = ContenidoBoveda {
        entradas: vec![e_remota],
        ..Default::default()
    };

    let fusionada = fusionar_bovedas(&boveda_local, &boveda_remota);

    assert_eq!(fusionada.entradas.len(), 1);
    let res = &fusionada.entradas[0];
    assert_eq!(res.contrasena, "claveAndroidNueva");
    assert_eq!(res.notas, "Actualizado desde el móvil");
    assert_eq!(res.revision, 2);
    assert_eq!(res.last_sync_hash, Some(hash_remoto));
}

#[test]
fn test_split_brain_preservando_credenciales_en_historial() {
    // 1. Estado base en rev 1
    let mut e_base = Entrada {
        id: "e-split-1".to_string(),
        titulo: "Servidor Produccion".to_string(),
        usuario: "admin".to_string(),
        contrasena: "claveInicial".to_string(),
        revision: 1,
        ..Default::default()
    };
    let hash_base = calcular_content_hash(&e_base);
    e_base.last_sync_hash = Some(hash_base);

    // Desktop cambió la contraseña a "claveDesktop" en t=3000 (rev 2)
    let mut e_local = e_base.clone();
    e_local.contrasena = "claveDesktop".to_string();
    e_local.modificada_en = 3000;
    e_local.revision = 2;

    // Android concurrentemente cambió la contraseña a "claveAndroid" en t=4000 (rev 2)
    let mut e_remota = e_base;
    e_remota.contrasena = "claveAndroid".to_string();
    e_remota.modificada_en = 4000; // Más reciente
    e_remota.revision = 2;

    let boveda_local = ContenidoBoveda {
        entradas: vec![e_local],
        ..Default::default()
    };
    let boveda_remota = ContenidoBoveda {
        entradas: vec![e_remota],
        ..Default::default()
    };

    let fusionada = fusionar_bovedas(&boveda_local, &boveda_remota);

    assert_eq!(fusionada.entradas.len(), 1);
    let res = &fusionada.entradas[0];

    // Android gana como principal por modificada_en (4000 > 3000)
    assert_eq!(res.contrasena, "claveAndroid");

    // La contraseña de Desktop NO se pierde: se preserva en el historial
    assert_eq!(res.historial_contrasenas.len(), 1);
    assert_eq!(res.historial_contrasenas[0].contrasena, "claveDesktop");

    // La revisión de bifurcación resuelta se incrementa a max(2, 2) + 1 = 3
    assert_eq!(res.revision, 3);

    // El last_sync_hash queda sellado con el hash de la entrada fusionada
    let nuevo_hash = calcular_content_hash(res);
    assert_eq!(res.last_sync_hash, Some(nuevo_hash));
}

#[test]
fn test_propagacion_papelera() {
    // 1. Entrada activa compartida en base rev 1
    let mut e_base = Entrada {
        id: "e-trash-1".to_string(),
        titulo: "Servicio Temporal".to_string(),
        usuario: "temp_user".to_string(),
        contrasena: "pass123".to_string(),
        revision: 1,
        ..Default::default()
    };
    let hash_base = calcular_content_hash(&e_base);
    e_base.last_sync_hash = Some(hash_base);

    // Desktop sigue con la entrada activa
    let boveda_local = ContenidoBoveda {
        entradas: vec![e_base.clone()],
        papelera: Vec::new(),
        ..Default::default()
    };

    // Android envió la entrada a la papelera (eliminada_en > 0, revision += 1)
    let mut e_remota_papelera = e_base;
    e_remota_papelera.eliminada_en = 5000;
    e_remota_papelera.modificada_en = 5000;
    e_remota_papelera.revision = 2;
    let hash_papelera = calcular_content_hash(&e_remota_papelera);

    let boveda_remota = ContenidoBoveda {
        entradas: Vec::new(),
        papelera: vec![e_remota_papelera],
        ..Default::default()
    };

    let fusionada = fusionar_bovedas(&boveda_local, &boveda_remota);

    // Ya no debe estar en activas
    assert_eq!(fusionada.entradas.len(), 0);

    // Debe propagarse a la papelera fusionada
    assert_eq!(fusionada.papelera.len(), 1);
    let res = &fusionada.papelera[0];
    assert_eq!(res.id, "e-trash-1");
    assert_eq!(res.eliminada_en, 5000);
    assert_eq!(res.revision, 2);
    assert_eq!(res.last_sync_hash, Some(hash_papelera));
}

#[test]
fn test_propagacion_alias_dispositivo_y_limpieza() {
    let boveda_local = ContenidoBoveda {
        alias_dispositivo: Some("ThinkPad (Linux Desktop)".to_string()),
        ..Default::default()
    };
    let boveda_remota = ContenidoBoveda {
        alias_dispositivo: Some("HONOR DNP-NX9".to_string()),
        ..Default::default()
    };

    let (fusionada, informe) = boveda_core::sync::fusionar_bovedas_con_informe(&boveda_local, &boveda_remota);

    // El informe debe capturar el dispositivo remoto
    assert_eq!(
        informe.dispositivo_remoto.as_deref(),
        Some("HONOR DNP-NX9")
    );

    // La bóveda fusionada resultante para guardar en disco debe tener alias_dispositivo: None
    assert_eq!(fusionada.alias_dispositivo, None);

    // El texto_resumen debe anteponer "con <dispositivo_remoto>: "
    let resumen = informe.texto_resumen();
    assert!(
        resumen.starts_with("con HONOR DNP-NX9: "),
        "Resumen inesperado: {resumen}"
    );

    // Serialización: alias_dispositivo: None no debe emitir el campo aliasDispositivo
    let json_str = serde_json::to_string(&fusionada).expect("Debe serializar");
    assert!(!json_str.contains("aliasDispositivo"));

    // Deserialización: si el JSON contiene "aliasDispositivo", se lee correctamente
    let json_con_alias = r#"{"version":1,"entradas":[],"papelera":[],"categorias":[],"identidades":[],"aliasDispositivo":"Xiaomi 22101320G"}"#;
    let boveda_deserializada: ContenidoBoveda = serde_json::from_str(json_con_alias).expect("Debe deserializar");
    assert_eq!(
        boveda_deserializada.alias_dispositivo.as_deref(),
        Some("Xiaomi 22101320G")
    );
}
