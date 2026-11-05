/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

use boveda_core::models::{
    CambioContrasena, ContenidoBoveda, DatosTarjetaBancaria, Entrada,
};
use boveda_core::sync::{
    calcular_content_hash, calcular_diff, de_entrada_nueva, fusionar_bovedas_con_informe,
    TipoCambioSync,
};

#[test]
fn test_entrada_nueva_enciende_campos_presentes() {
    let entrada = Entrada {
        id: "id-1".into(),
        titulo: "GitHub".into(),
        usuario: "octocat".into(),
        contrasena: "secret123".into(),
        urls: vec![
            "https://github.com".into(),
            "androidapp://com.github.android".into(),
        ],
        notas: "Token personal guardado".into(),
        secreto_totp: Some("JBSWY3DPEHPK3PXP".into()),
        ..Default::default()
    };

    let mascara = de_entrada_nueva(&entrada);

    assert!(mascara.usuario);
    assert!(mascara.clave);
    assert!(mascara.totp);
    assert!(mascara.web);
    assert!(mascara.app);
    assert!(mascara.notas);
    assert!(!mascara.passkey);
    assert!(!mascara.submodelos);
    assert!(!mascara.campos_extra);
    assert!(mascara.hay_cambios());
}

#[test]
fn test_clave_se_ilumina_con_cambio_en_historial_aunque_contrasena_sea_igual() {
    let previa = Entrada {
        id: "id-2".into(),
        titulo: "Servidor".into(),
        contrasena: "claveGanadora".into(),
        historial_contrasenas: vec![],
        ..Default::default()
    };

    let nueva = Entrada {
        id: "id-2".into(),
        titulo: "Servidor".into(),
        contrasena: "claveGanadora".into(),
        historial_contrasenas: vec![CambioContrasena {
            contrasena: "claveAntiguaRemota".into(),
            cambiada_en: 1000,
        }],
        ..Default::default()
    };

    let mascara = calcular_diff(&previa, &nueva);

    assert!(
        mascara.clave,
        "La clave debe iluminarse si el historial absorbió una credencial alternativa"
    );
    assert!(!mascara.usuario);
    assert!(!mascara.totp);
    assert!(mascara.hay_cambios());
}

#[test]
fn test_diff_urls_distingue_web_de_app() {
    let previa = Entrada {
        id: "id-3".into(),
        titulo: "Amazon".into(),
        urls: vec!["https://amazon.com".into()],
        ..Default::default()
    };

    let nueva_con_app = Entrada {
        id: "id-3".into(),
        titulo: "Amazon".into(),
        urls: vec![
            "https://amazon.com".into(),
            "androidapp://com.amazon.mShop.android.shopping".into(),
        ],
        ..Default::default()
    };

    let mascara = calcular_diff(&previa, &nueva_con_app);

    assert!(!mascara.web);
    assert!(mascara.app);
    assert!(!mascara.clave);
    assert!(mascara.hay_cambios());
}

#[test]
fn test_diff_submodelos_nativos() {
    let previa = Entrada {
        id: "id-4".into(),
        titulo: "Tarjeta Visa".into(),
        datos_tarjeta: Some(DatosTarjetaBancaria {
            titular: "Juan Perez".into(),
            numero: "4111111111111111".into(),
            ..Default::default()
        }),
        ..Default::default()
    };

    let nueva = Entrada {
        id: "id-4".into(),
        titulo: "Tarjeta Visa".into(),
        datos_tarjeta: Some(DatosTarjetaBancaria {
            titular: "Juan Perez".into(),
            numero: "4111111111112222".into(),
            ..Default::default()
        }),
        ..Default::default()
    };

    let mascara = calcular_diff(&previa, &nueva);

    assert!(mascara.submodelos);
    assert!(!mascara.clave);
    assert!(!mascara.usuario);
    assert!(mascara.hay_cambios());
}

#[test]
fn test_transicion_papelera_con_modificacion_de_campos_simultanea() {
    let en_papelera_local = Entrada {
        id: "id-5".into(),
        titulo: "Netflix".into(),
        contrasena: "claveVieja".into(),
        eliminada_en: 1700000000,
        ..Default::default()
    };

    let restaurada_remota = Entrada {
        id: "id-5".into(),
        titulo: "Netflix".into(),
        contrasena: "claveNueva".into(),
        eliminada_en: 0,
        ..Default::default()
    };

    let boveda_local = ContenidoBoveda {
        papelera: vec![en_papelera_local],
        ..Default::default()
    };
    let boveda_remota = ContenidoBoveda {
        entradas: vec![restaurada_remota],
        ..Default::default()
    };

    let (boveda_res, informe) = fusionar_bovedas_con_informe(&boveda_local, &boveda_remota);

    assert_eq!(boveda_res.entradas.len(), 1);
    assert_eq!(informe.nuevas.len(), 0);
    assert_eq!(informe.actualizadas.len(), 0);
    assert_eq!(informe.papelera.len(), 1);

    let item = &informe.papelera[0];
    assert_eq!(item.tipo_cambio, TipoCambioSync::RestauradaDePapelera);
    assert!(
        item.mascara.clave,
        "La máscara debe iluminar el campo clave que cambió al restaurarse"
    );
}

#[test]
fn test_entradas_sin_cambios_conteo_y_resumen() {
    let mut entrada_base = Entrada {
        id: "id-6".into(),
        titulo: "Spotify".into(),
        usuario: "user@spotify.com".into(),
        contrasena: "mismaclave".into(),
        revision: 1,
        modificada_en: 2000,
        ..Default::default()
    };
    let hash_base = calcular_content_hash(&entrada_base);
    entrada_base.last_sync_hash = Some(hash_base);

    let local = Entrada {
        revision: 2,
        modificada_en: 3000,
        ..entrada_base.clone()
    };
    let remota = Entrada {
        revision: 2,
        modificada_en: 2500,
        ..entrada_base
    };

    let boveda_local = ContenidoBoveda {
        entradas: vec![local],
        ..Default::default()
    };
    let boveda_remota = ContenidoBoveda {
        entradas: vec![remota],
        ..Default::default()
    };

    let (_res, informe) = fusionar_bovedas_con_informe(&boveda_local, &boveda_remota);

    assert_eq!(informe.nuevas.len(), 0);
    assert_eq!(informe.actualizadas.len(), 0);
    assert_eq!(informe.papelera.len(), 0);
    assert_eq!(informe.sin_cambios_count, 1);
    assert!(!informe.hay_cambios());
    assert_eq!(
        informe.texto_resumen(),
        "Bóvedas idénticas (sin cambios, 1 al día)"
    );
}

#[test]
fn test_texto_resumen_multiples_cambios() {
    let mut a = ContenidoBoveda::default();
    a.entradas.push(Entrada {
        id: "1".into(),
        titulo: "Google".into(),
        usuario: "u1".into(),
        ..Default::default()
    });

    let mut b = ContenidoBoveda::default();
    b.entradas.push(Entrada {
        id: "1".into(),
        titulo: "Google".into(),
        usuario: "u2_editado".into(),
        revision: 2,
        ..Default::default()
    });
    b.entradas.push(Entrada {
        id: "2".into(),
        titulo: "GitHub".into(),
        contrasena: "pass".into(),
        ..Default::default()
    });

    let (_boveda, informe) = fusionar_bovedas_con_informe(&a, &b);

    assert_eq!(informe.nuevas.len(), 1);
    assert_eq!(informe.actualizadas.len(), 1);
    assert!(informe.hay_cambios());

    let resumen = informe.texto_resumen();
    assert!(resumen.contains("1 nueva(s) («GitHub»)"));
    assert!(resumen.contains("1 actualizada(s) («Google»)"));
}
