/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use boveda_core::models::{ContenidoBoveda, Entrada};
use boveda_core::sync::{
    calcular_content_hash, derivar_clave_publica_x25519, ejecutar_cliente_sync_p2p,
    escribir_trama, leer_trama, ConfiguracionClienteP2P, ConfiguracionServidorP2P, ServidorSyncP2P,
};

fn crear_entrada_prueba(id: &str, titulo: &str, usuario: &str, rev: u64, modif: i64) -> Entrada {
    Entrada {
        id: id.to_string(),
        titulo: titulo.to_string(),
        usuario: usuario.to_string(),
        contrasena: "pass123".to_string(),
        revision: rev,
        modificada_en: modif,
        ..Default::default()
    }
}

#[test]
fn test_sincronizacion_p2p_end_to_end_exitosa() {
    // Claves del servidor (Desktop)
    let sk_desktop = [0x11u8; 32];
    let pk_desktop = derivar_clave_publica_x25519(&sk_desktop);

    // Claves del cliente (Android)
    let sk_android = [0x22u8; 32];
    let pk_android = derivar_clave_publica_x25519(&sk_android);

    // Token PSK compartido del código QR
    let psk_token = [0x77u8; 32];

    // Bóveda Desktop (local): Entrada A y Entrada C v1
    let mut boveda_desktop = ContenidoBoveda::default();
    boveda_desktop.entradas.push(crear_entrada_prueba(
        "uuid-a",
        "Servicio A",
        "user_a",
        1,
        1000,
    ));
    boveda_desktop.entradas.push(crear_entrada_prueba(
        "uuid-c",
        "Servicio C",
        "user_c_v1",
        1,
        1000,
    ));

    // Bóveda Android (remota): Entrada B y Entrada C v2 (que avanzó desde v1)
    let mut boveda_android = ContenidoBoveda::default();
    boveda_android.entradas.push(crear_entrada_prueba(
        "uuid-b",
        "Servicio B",
        "user_b",
        1,
        1000,
    ));

    let mut entrada_c_android = crear_entrada_prueba(
        "uuid-c",
        "Servicio C",
        "user_c_v2",
        2,
        2000,
    );
    // Su last_sync_hash apunta a la v1 que tenía Desktop
    let entrada_c_desktop = &boveda_desktop.entradas[1];
    entrada_c_android.last_sync_hash = Some(calcular_content_hash(entrada_c_desktop));
    boveda_android.entradas.push(entrada_c_android);

    // Configurar servidor en puerto efímero (0)
    let config_servidor = ConfiguracionServidorP2P {
        sk_desktop,
        psk_token,
        puerto: 0,
        timeout_segundos: 10,
    };

    let servidor = ServidorSyncP2P::bind(config_servidor).expect("Fallo al iniciar servidor");
    let puerto = servidor.puerto_local().expect("Fallo obteniendo puerto efímero");

    let commit_invocado = Arc::new(AtomicBool::new(false));
    let commit_flag_cloned = Arc::clone(&commit_invocado);

    // Hilo del Servidor (Desktop)
    let handle_servidor = thread::spawn(move || {
        servidor.atender_sincronizacion(&boveda_desktop, move |_boveda_fusionada| {
            commit_flag_cloned.store(true, Ordering::SeqCst);
            Ok(())
        })
    });

    // Pequeño retardo para asegurar que el socket está en escucha
    thread::sleep(Duration::from_millis(50));

    // Hilo del Cliente (Android)
    let config_cliente = ConfiguracionClienteP2P {
        sk_cliente: sk_android,
        pk_servidor: pk_desktop,
        psk_token,
        host: "127.0.0.1".to_string(),
        puerto,
        timeout_segundos: 10,
    };

    let boveda_cliente_result =
        ejecutar_cliente_sync_p2p(&config_cliente, &boveda_android).expect("Cliente P2P falló");

    let res_servidor = handle_servidor
        .join()
        .expect("Pánico en hilo de servidor")
        .expect("Servidor P2P falló");

    // Verificaciones
    assert!(
        commit_invocado.load(Ordering::SeqCst),
        "El callback de commit atómico debe haberse ejecutado"
    );
    assert_eq!(res_servidor.ack_recibido, "OK");
    assert_eq!(res_servidor.pk_android, pk_android);

    // Ambas bóvedas deben contener exactamente 3 entradas
    assert_eq!(res_servidor.boveda_fusionada.entradas.len(), 3);
    assert_eq!(boveda_cliente_result.entradas.len(), 3);

    // Las entradas deben estar presentes y coincidentes
    let ids_servidor: Vec<&str> = res_servidor
        .boveda_fusionada
        .entradas
        .iter()
        .map(|e| e.id.as_str())
        .collect();
    assert!(ids_servidor.contains(&"uuid-a"));
    assert!(ids_servidor.contains(&"uuid-b"));
    assert!(ids_servidor.contains(&"uuid-c"));

    // Entrada C debe ser la versión 2 (FastForwardRemoto)
    let entrada_c = res_servidor
        .boveda_fusionada
        .entradas
        .iter()
        .find(|e| e.id == "uuid-c")
        .unwrap();
    assert_eq!(entrada_c.revision, 2);
    assert_eq!(entrada_c.usuario, "user_c_v2");
    assert!(entrada_c.last_sync_hash.is_some());
}

#[test]
fn test_sincronizacion_p2p_split_brain_concurrente() {
    let sk_desktop = [0x55u8; 32];
    let pk_desktop = derivar_clave_publica_x25519(&sk_desktop);
    let sk_android = [0x66u8; 32];
    let psk_token = [0x88u8; 32];

    // Entrada común editada concurrentemente en ambos dispositivos (rev 1 vs rev 1)
    let e_desktop = crear_entrada_prueba("uuid-sync", "Servicio X", "user_desk", 1, 1000);
    let mut boveda_desktop = ContenidoBoveda::default();
    boveda_desktop.entradas.push(e_desktop);

    let e_android = crear_entrada_prueba("uuid-sync", "Servicio X", "user_android", 1, 1500);
    let mut boveda_android = ContenidoBoveda::default();
    boveda_android.entradas.push(e_android);

    let config_servidor = ConfiguracionServidorP2P {
        sk_desktop,
        psk_token,
        puerto: 0,
        timeout_segundos: 10,
    };
    let servidor = ServidorSyncP2P::bind(config_servidor).expect("Fallo al iniciar servidor");
    let puerto = servidor.puerto_local().expect("Fallo puerto");

    let handle_servidor = thread::spawn(move || {
        servidor.atender_sincronizacion(&boveda_desktop, |_| Ok(()))
    });

    thread::sleep(Duration::from_millis(50));

    let config_cliente = ConfiguracionClienteP2P {
        sk_cliente: sk_android,
        pk_servidor: pk_desktop,
        psk_token,
        host: "127.0.0.1".to_string(),
        puerto,
        timeout_segundos: 10,
    };

    let boveda_cliente =
        ejecutar_cliente_sync_p2p(&config_cliente, &boveda_android).expect("Cliente P2P falló");
    let res_servidor = handle_servidor.join().unwrap().unwrap();

    // Verificación de SplitBrain: revision = 1.max(1) + 1 = 2
    // Modificada_en de Android (1500) > Desktop (1000), prevalece usuario "user_android"
    assert_eq!(res_servidor.boveda_fusionada.entradas.len(), 1);
    assert_eq!(boveda_cliente.entradas.len(), 1);

    let fusionada = &res_servidor.boveda_fusionada.entradas[0];
    assert_eq!(fusionada.revision, 2);
    assert_eq!(fusionada.usuario, "user_android");
    assert!(fusionada.last_sync_hash.is_some());
}

#[test]
fn test_rechazo_handshake_con_psk_invalida() {
    let sk_desktop = [0x33u8; 32];
    let pk_desktop = derivar_clave_publica_x25519(&sk_desktop);

    let sk_android = [0x44u8; 32];
    let psk_correcta = [0xAAu8; 32];
    let psk_incorrecta = [0xBBu8; 32];

    let config_servidor = ConfiguracionServidorP2P {
        sk_desktop,
        psk_token: psk_correcta,
        puerto: 0,
        timeout_segundos: 5,
    };

    let servidor = ServidorSyncP2P::bind(config_servidor).expect("Fallo al iniciar servidor");
    let puerto = servidor.puerto_local().expect("Fallo obteniendo puerto efímero");

    let boveda_local = ContenidoBoveda::default();

    let handle_servidor = thread::spawn(move || {
        servidor.atender_sincronizacion(&boveda_local, |_| Ok(()))
    });

    thread::sleep(Duration::from_millis(50));

    // Cliente se conecta con token PSK alterado
    let config_cliente_invalido = ConfiguracionClienteP2P {
        sk_cliente: sk_android,
        pk_servidor: pk_desktop,
        psk_token: psk_incorrecta,
        host: "127.0.0.1".to_string(),
        puerto,
        timeout_segundos: 5,
    };

    let boveda_android = ContenidoBoveda::default();
    let res_cliente = ejecutar_cliente_sync_p2p(&config_cliente_invalido, &boveda_android);

    assert!(
        res_cliente.is_err(),
        "El cliente debe rechazar o ser rechazado por PSK inválida"
    );

    let res_servidor = handle_servidor.join().expect("Pánico en servidor");
    assert!(
        res_servidor.is_err(),
        "El servidor debe fallar el handshake ante un cliente no autenticado"
    );
}

#[test]
fn test_wire_framing_limite_tamano() {
    use std::io::Cursor;

    // Crear trama con longitud declarada de 50MB (mayor al límite por defecto de 32MB)
    let mut buffer = Vec::new();
    let longitud_excesiva: u32 = 50 * 1024 * 1024;
    buffer.extend_from_slice(&longitud_excesiva.to_le_bytes());

    let mut cursor = Cursor::new(buffer);
    let res = leer_trama(&mut cursor, 32 * 1024 * 1024);

    assert!(res.is_err());
    assert!(res
        .unwrap_err()
        .contains("exceder tamaño máximo permitido"));
}

#[test]
fn test_wire_framing_escritura_y_lectura() {
    use std::io::Cursor;

    let mensaje_original = b"Mensaje de prueba de trama segura P2P";
    let mut buffer = Vec::new();

    escribir_trama(&mut buffer, mensaje_original).expect("Fallo escribiendo trama");
    assert_eq!(buffer.len(), 4 + mensaje_original.len());

    let mut cursor = Cursor::new(buffer);
    let mensaje_recuperado =
        leer_trama(&mut cursor, 1024).expect("Fallo leyendo trama");

    assert_eq!(mensaje_recuperado, mensaje_original);
}

#[test]
fn test_confirmacion_par_rechazada() {
    let sk_desktop = [0x71u8; 32];
    let pk_desktop = derivar_clave_publica_x25519(&sk_desktop);
    let sk_android = [0x72u8; 32];
    let psk_token = [0x73u8; 32];

    let boveda_desktop = ContenidoBoveda::default();
    let boveda_android = ContenidoBoveda {
        alias_dispositivo: Some("Honor Magic 6 Pro".to_string()),
        ..Default::default()
    };

    let config_servidor = ConfiguracionServidorP2P {
        sk_desktop,
        psk_token,
        puerto: 0,
        timeout_segundos: 10,
    };
    let servidor = ServidorSyncP2P::bind(config_servidor).expect("Fallo bind");
    let puerto = servidor.puerto_local().expect("Fallo puerto");

    let cancel_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let commit_ejecutado = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let commit_clone = std::sync::Arc::clone(&commit_ejecutado);

    let handle_servidor = std::thread::spawn(move || {
        servidor.atender_sincronizacion_cancelable_con_confirmacion(
            &boveda_desktop,
            |_| {
                commit_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            },
            cancel_flag,
            |alias, _huella| {
                assert_eq!(alias, "Honor Magic 6 Pro");
                false // Rechazar
            },
        )
    });

    let config_cliente = ConfiguracionClienteP2P {
        sk_cliente: sk_android,
        pk_servidor: pk_desktop,
        psk_token,
        host: "127.0.0.1".to_string(),
        puerto,
        timeout_segundos: 10,
    };

    let _ = ejecutar_cliente_sync_p2p(&config_cliente, &boveda_android);
    let res_servidor = handle_servidor.join().expect("Pánico servidor");

    assert!(res_servidor.is_err());
    assert!(res_servidor.unwrap_err().contains("rechazada por el usuario"));
    assert!(!commit_ejecutado.load(std::sync::atomic::Ordering::SeqCst));
}

#[test]
fn test_confirmacion_par_aceptada_y_saludo() {
    let sk_desktop = [0x81u8; 32];
    let pk_desktop = derivar_clave_publica_x25519(&sk_desktop);
    let sk_android = [0x82u8; 32];
    let psk_token = [0x83u8; 32];

    let boveda_desktop = ContenidoBoveda::default();
    let boveda_android = ContenidoBoveda {
        alias_dispositivo: Some("Galaxy S24".to_string()),
        ..Default::default()
    };

    let config_servidor = ConfiguracionServidorP2P {
        sk_desktop,
        psk_token,
        puerto: 0,
        timeout_segundos: 10,
    };
    let servidor = ServidorSyncP2P::bind(config_servidor).expect("Fallo bind");
    let puerto = servidor.puerto_local().expect("Fallo puerto");

    let cancel_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let confirmado = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let confirmado_clone = std::sync::Arc::clone(&confirmado);

    let handle_servidor = std::thread::spawn(move || {
        servidor.atender_sincronizacion_cancelable_con_confirmacion(
            &boveda_desktop,
            |_| Ok(()),
            cancel_flag,
            move |alias, huella| {
                assert_eq!(alias, "Galaxy S24");
                assert_eq!(huella.len(), 17); // XX:XX:XX:XX:XX:XX
                confirmado_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                true // Aceptar
            },
        )
    });

    let config_cliente = ConfiguracionClienteP2P {
        sk_cliente: sk_android,
        pk_servidor: pk_desktop,
        psk_token,
        host: "127.0.0.1".to_string(),
        puerto,
        timeout_segundos: 10,
    };

    let res_cliente = ejecutar_cliente_sync_p2p(&config_cliente, &boveda_android)
        .expect("Cliente debe sincronizar");
    let res_servidor = handle_servidor.join().expect("Pánico servidor")
        .expect("Servidor debe sincronizar");

    assert!(confirmado.load(std::sync::atomic::Ordering::SeqCst));
    assert_eq!(res_servidor.informe_sync.dispositivo_remoto.as_deref(), Some("Galaxy S24"));
    assert_eq!(res_cliente.alias_dispositivo.as_deref(), Some(boveda_core::sync::obtener_nombre_desktop().as_str()));
}
