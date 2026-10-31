/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Herramienta de línea de comandos para Bóveda Local Desktop.
//! Soporta inspección, desbloqueo, fusión, importación, exportación y configuración.

use boveda_core::config::AppConfigDesktop;
use boveda_core::crypto::{
    desempaquetar_boveda, empaquetar_boveda, escribir_atomico, CabeceraBvda, PerfilArgon2,
    TAM_CABECERA, TAM_TAG_GCM,
};
use boveda_core::models::{
    Categoria, ContenidoBoveda, DatosContacto, DatosCuentaBancaria, DatosCriptoWallet,
    DatosDocumentoIdentidad, DatosRedWifi, DatosServidorSsh, DatosTarjetaBancaria, Entrada,
    Perfil, TipoEntrada,
};
use boveda_core::sync::{
    exportar_csv, exportar_json_plano, fusionar_bovedas, importar_bitwarden_json, importar_csv,
};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::Path;

fn main() {
    println!("==================================================");
    println!("  Bóveda Local Desktop (CLI Criptográfica en Rust)");
    println!("  Compatibilidad nativa con archivos .bvda");
    println!("==================================================\n");

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        imprimir_ayuda();
        return;
    }

    match args[1].as_str() {
        "inspect" => {
            if args.len() < 3 {
                eprintln!("Error: Debes especificar la ruta al archivo .bvda");
                eprintln!("Uso: boveda-cli inspect <ruta_archivo.bvda>");
                return;
            }
            inspeccionar_cabecera(&args[2]);
        }
        "unlock" => {
            if args.len() < 3 {
                eprintln!("Error: Debes especificar la ruta al archivo .bvda");
                eprintln!("Uso: boveda-cli unlock <ruta_archivo.bvda> [--password <clave>]");
                return;
            }
            let pass_arg = extraer_parametro(&args, "--password").or_else(|| extraer_parametro(&args, "-p"));
            desbloquear_boveda(&args[2], pass_arg.as_deref());
        }
        "create" => {
            if args.len() < 3 {
                eprintln!("Error: Debes especificar la ruta de destino para el archivo .bvda");
                eprintln!("Uso: boveda-cli create <ruta_destino.bvda> [--password <clave>]");
                return;
            }
            let pass_arg = extraer_parametro(&args, "--password").or_else(|| extraer_parametro(&args, "-p"));
            crear_boveda_demo(&args[2], pass_arg.as_deref());
        }
        "merge" => {
            if args.len() < 4 {
                eprintln!("Uso: boveda-cli merge <boveda_a.bvda> <boveda_b.bvda> -o <resultado.bvda> [--password <clave>]");
                return;
            }
            let ruta_a = &args[2];
            let ruta_b = &args[3];
            let salida = match extraer_parametro(&args, "-o").or_else(|| extraer_parametro(&args, "--output")) {
                Some(s) => s,
                None => {
                    eprintln!("Error: Debes especificar la ruta de salida con -o <resultado.bvda>");
                    return;
                }
            };
            let pass_arg = extraer_parametro(&args, "--password").or_else(|| extraer_parametro(&args, "-p"));
            fusionar_archivos(ruta_a, ruta_b, &salida, pass_arg.as_deref());
        }
        "import" => {
            if args.len() < 4 {
                eprintln!("Uso:");
                eprintln!("  boveda-cli import bitwarden <archivo.json> -o <resultado.bvda> [--password <clave>]");
                eprintln!("  boveda-cli import csv       <archivo.csv>  -o <resultado.bvda> [--password <clave>]");
                return;
            }
            let formato = &args[2];
            let origen = &args[3];
            let salida = match extraer_parametro(&args, "-o").or_else(|| extraer_parametro(&args, "--output")) {
                Some(s) => s,
                None => {
                    eprintln!("Error: Debes especificar la ruta de destino .bvda con -o <resultado.bvda>");
                    return;
                }
            };
            let pass_arg = extraer_parametro(&args, "--password").or_else(|| extraer_parametro(&args, "-p"));
            importar_archivo(formato, origen, &salida, pass_arg.as_deref());
        }
        "export" => {
            if args.len() < 4 {
                eprintln!("Uso:");
                eprintln!("  boveda-cli export csv  <archivo.bvda> [-o <salida.csv>] [--password <clave>]");
                eprintln!("  boveda-cli export json <archivo.bvda> [-o <salida.json>] [--password <clave>]");
                return;
            }
            let formato = &args[2];
            let boveda_ruta = &args[3];
            let salida_opt = extraer_parametro(&args, "-o").or_else(|| extraer_parametro(&args, "--output"));
            let pass_arg = extraer_parametro(&args, "--password").or_else(|| extraer_parametro(&args, "-p"));
            exportar_archivo(formato, boveda_ruta, salida_opt.as_deref(), pass_arg.as_deref());
        }
        "config" => {
            gestionar_configuracion(&args[2..]);
        }
        "help" | "--help" | "-h" => {
            imprimir_ayuda();
        }
        cmd => {
            eprintln!("Comando desconocido: '{}'", cmd);
            imprimir_ayuda();
        }
    }
}

fn extraer_parametro(args: &[String], flag: &str) -> Option<String> {
    if let Some(pos) = args.iter().position(|a| a == flag) {
        if pos + 1 < args.len() {
            return Some(args[pos + 1].clone());
        }
    }
    None
}

fn imprimir_ayuda() {
    println!("Uso de boveda-cli:");
    println!("  boveda-cli inspect <archivo.bvda>                               Inspecciona cabecera AAD y parámetros KDF");
    println!("  boveda-cli unlock  <archivo.bvda> [-p <clave>]                 Desbloquea e inspecciona las entradas");
    println!("  boveda-cli create  <archivo.bvda> [-p <clave>]                 Crea una nueva bóveda demo cifrada");
    println!("  boveda-cli merge   <a.bvda> <b.bvda> -o <res.bvda> [-p <clave>] Fusión aditiva de dos bóvedas sin pérdida");
    println!("  boveda-cli import  bitwarden|csv <origen> -o <res.bvda>        Importa credenciales desde Bitwarden o CSV");
    println!("  boveda-cli export  csv|json <archivo.bvda> [-o <salida>]       Exporta credenciales descifradas a CSV/JSON");
    println!("  boveda-cli config  list|get|set                                Gestiona los ajustes locales del cliente");
    println!();
}

fn leer_contrasena(prompt: &str) -> Result<String, io::Error> {
    print!("{}", prompt);
    let _ = io::stdout().flush();

    if io::stdin().is_terminal() {
        rpassword::prompt_password("").map_err(io::Error::other)
    } else {
        let mut linea = String::new();
        io::stdin().read_line(&mut linea)?;
        if linea.ends_with('\n') {
            linea.pop();
            if linea.ends_with('\r') {
                linea.pop();
            }
        }
        Ok(linea)
    }
}

fn inspeccionar_cabecera(ruta: &str) {
    let bytes = match fs::read(ruta) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error al leer archivo '{}': {}", ruta, e);
            return;
        }
    };

    match CabeceraBvda::parsear(&bytes) {
        Ok(cabecera) => {
            println!("🔒 Cabecera .bvda válida detectada:");
            println!("  Versión de archivo:   {}", cabecera.version);
            println!("  Memoria Argon2id:     {} KiB ({} MiB)", cabecera.memory_kib, cabecera.memory_kib / 1024);
            println!("  Iteraciones Argon2id: {}", cabecera.iterations);
            println!("  Paralelismo:          {} lanes", cabecera.parallelism);
            println!("  Longitud de clave:    {} bytes (256 bits)", cabecera.hash_length);
            println!("  Salt CSPRNG (hex):    {}", hex_fmt(&cabecera.salt));
            println!("  Nonce AES-GCM (hex):  {}", hex_fmt(&cabecera.nonce));
            let cuerpo_len = bytes.len().saturating_sub(TAM_CABECERA + TAM_TAG_GCM);
            println!("  Tamaño Ciphertext:    {} bytes", cuerpo_len);
            println!("  Tamaño Tag GCM:       {} bytes", TAM_TAG_GCM);
            println!("  Tamaño Total Archivo: {} bytes ({:.2} KiB)", bytes.len(), bytes.len() as f64 / 1024.0);
        }
        Err(e) => {
            eprintln!("Fallo al validar la cabecera del archivo: {}", e);
        }
    }
}

fn desbloquear_boveda(ruta: &str, password_opt: Option<&str>) {
    let bytes = match fs::read(ruta) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error al leer archivo '{}': {}", ruta, e);
            return;
        }
    };

    let password = match password_opt {
        Some(p) => p.to_string(),
        None => match leer_contrasena("🔑 Introduce la contraseña maestra: ") {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error al leer contraseña: {}", e);
                return;
            }
        },
    };

    println!("\n⏳ Derivando clave Argon2id y descifrando AES-256-GCM...");
    let contenido = match desempaquetar_boveda(&bytes, password.as_bytes()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("\n❌ Error al desbloquear la bóveda: {}", e);
            return;
        }
    };

    println!("✅ Bóveda desbloqueada con éxito!\n");
    println!("📊 Resumen de la Bóveda (Versión de esquema: {})", contenido.version);
    println!("--------------------------------------------------");
    println!("  Total Entradas Activas: {}", contenido.entradas.len());
    println!("  Categorías / Carpetas:  {}", contenido.categorias.len());
    println!("  Perfiles de Identidad:  {}", contenido.perfiles.len());
    println!("  Entradas en Papelera:   {}", contenido.papelera.len());

    let mut desglose_tipos: HashMap<TipoEntrada, usize> = HashMap::new();
    for entrada in &contenido.entradas {
        *desglose_tipos.entry(entrada.tipo).or_insert(0) += 1;
    }

    println!("\n📁 Desglose por Tipo de Contenido:");
    for (tipo, conteo) in desglose_tipos {
        println!("  - {:<22}: {}", tipo.etiqueta(), conteo);
    }

    if let Some(cfg) = &contenido.configuracion {
        println!("\n⚙️ Configuración Portable de la Bóveda:");
        println!("  - Perfil Argon2:          {}", cfg.perfil_argon2);
        println!("  - Auto-bloqueo:           {} segundos", cfg.auto_bloqueo_segundos);
        println!("  - Limpieza Portapapeles:  {} segundos", cfg.portapapeles_segundos);
        println!("  - Criterio de Ordenación: {}", cfg.criterio_ordenacion);
    }

    println!("\n📋 Listado de Entradas:");
    println!("{:<36} | {:<18} | {:<25} | {:<25} | {:<4}", "ID", "Tipo", "Título", "Usuario / Detalle", "TOTP");
    println!("{}", "-".repeat(116));

    for e in &contenido.entradas {
        let detalle = match e.tipo {
            TipoEntrada::Login => e.usuario.clone(),
            TipoEntrada::Tarjeta => e.datos_tarjeta.as_ref().map(|t| {
                let ultimos: String = t.numero.chars().rev().take(4).collect::<Vec<_>>().into_iter().rev().collect();
                format!("PAN: ...{}", ultimos)
            }).unwrap_or_default(),
            TipoEntrada::Wifi => e.datos_wifi.as_ref().map(|w| format!("SSID: {}", w.ssid)).unwrap_or_default(),
            TipoEntrada::CuentaBancaria => e.datos_cuenta_bancaria.as_ref().map(|b| format!("Banco: {}", b.banco)).unwrap_or_default(),
            TipoEntrada::Servidor => e.datos_servidor.as_ref().map(|s| format!("{}@{}", s.usuario, s.host)).unwrap_or_default(),
            TipoEntrada::Wallet => e.datos_wallet.as_ref().map(|w| format!("Red: {}", w.red)).unwrap_or_default(),
            TipoEntrada::Contacto => e.datos_contacto.as_ref().map(|c| c.correo.clone()).unwrap_or_default(),
            _ => e.usuario.clone(),
        };

        let totp_badge = if e.tiene_totp() { "Sí" } else { "No" };
        let titulo_trunc = if e.titulo.len() > 24 { format!("{}...", &e.titulo[..21]) } else { e.titulo.clone() };
        let detalle_trunc = if detalle.len() > 24 { format!("{}...", &detalle[..21]) } else { detalle };

        println!("{:<36} | {:<18} | {:<25} | {:<25} | {:<4}", e.id, e.tipo.etiqueta(), titulo_trunc, detalle_trunc, totp_badge);
    }
    println!();
}

fn fusionar_archivos(ruta_a: &str, ruta_b: &str, ruta_salida: &str, password_opt: Option<&str>) {
    let bytes_a = match fs::read(ruta_a) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error al leer bóveda A '{}': {}", ruta_a, e);
            return;
        }
    };
    let bytes_b = match fs::read(ruta_b) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error al leer bóveda B '{}': {}", ruta_b, e);
            return;
        }
    };

    let password = match password_opt {
        Some(p) => p.to_string(),
        None => match leer_contrasena("🔑 Contraseña maestra de las bóvedas: ") {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error al leer contraseña: {}", e);
                return;
            }
        },
    };

    println!("⏳ Desbloqueando bóveda A...");
    let boveda_a = match desempaquetar_boveda(&bytes_a, password.as_bytes()) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error al descifrar bóveda A: {}", e);
            return;
        }
    };

    println!("⏳ Desbloqueando bóveda B...");
    let boveda_b = match desempaquetar_boveda(&bytes_b, password.as_bytes()) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error al descifrar bóveda B: {}", e);
            return;
        }
    };

    println!("🔀 Aplicando algoritmo de unificación sin pérdida (Lossless Merge)...");
    let fusionada = fusionar_bovedas(&boveda_a, &boveda_b);

    println!("🔒 Cifrando bóveda unificada en '{}'...", ruta_salida);
    let bytes_cifrados = match empaquetar_boveda(&fusionada, password.as_bytes(), PerfilArgon2::Estandar) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error al empaquetar bóveda unificada: {}", e);
            return;
        }
    };

    if let Err(e) = escribir_atomico(Path::new(ruta_salida), &bytes_cifrados) {
        eprintln!("Error al escribir archivo unificado: {}", e);
        return;
    }

    println!("✅ Fusión completada con éxito!");
    println!("   Total entradas resultantes: {} (A tenía {}, B tenía {})", fusionada.entradas.len(), boveda_a.entradas.len(), boveda_b.entradas.len());
    println!("   Categorías resultantes:     {}", fusionada.categorias.len());
    println!("   Perfiles resultantes:       {}", fusionada.perfiles.len());
}

fn importar_archivo(formato: &str, origen: &str, salida: &str, password_opt: Option<&str>) {
    let bytes_origen = match fs::read(origen) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error al leer archivo origen '{}': {}", origen, e);
            return;
        }
    };

    let boveda = match formato.to_lowercase().as_str() {
        "bitwarden" => match importar_bitwarden_json(&bytes_origen) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Error al importar JSON de Bitwarden: {}", e);
                return;
            }
        },
        "csv" => {
            let entradas = match importar_csv(&bytes_origen, None) {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("Error al importar CSV: {}", e);
                    return;
                }
            };
            ContenidoBoveda {
                entradas,
                ..Default::default()
            }
        }
        otro => {
            eprintln!("Formato no soportado: '{}'. Usa 'bitwarden' o 'csv'.", otro);
            return;
        }
    };

    let password = match password_opt {
        Some(p) => p.to_string(),
        None => match leer_contrasena("🔑 Contraseña maestra para la nueva bóveda: ") {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error al leer contraseña: {}", e);
                return;
            }
        },
    };

    println!("⏳ Cifrando {} entradas en '{}'...", boveda.entradas.len(), salida);
    let bytes_cifrados = match empaquetar_boveda(&boveda, password.as_bytes(), PerfilArgon2::Estandar) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error al empaquetar bóveda: {}", e);
            return;
        }
    };

    if let Err(e) = escribir_atomico(Path::new(salida), &bytes_cifrados) {
        eprintln!("Error al escribir archivo de bóveda: {}", e);
        return;
    }

    println!("✅ Importación completada: {} entradas empaquetadas en '{}'.", boveda.entradas.len(), salida);
}

fn exportar_archivo(formato: &str, boveda_ruta: &str, salida_opt: Option<&str>, password_opt: Option<&str>) {
    let bytes = match fs::read(boveda_ruta) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error al leer bóveda '{}': {}", boveda_ruta, e);
            return;
        }
    };

    let password = match password_opt {
        Some(p) => p.to_string(),
        None => match leer_contrasena("🔑 Contraseña maestra de la bóveda: ") {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error al leer contraseña: {}", e);
                return;
            }
        },
    };

    println!("⏳ Descifrando bóveda...");
    let boveda = match desempaquetar_boveda(&bytes, password.as_bytes()) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error al desbloquear bóveda: {}", e);
            return;
        }
    };

    let contenido_exportado = match formato.to_lowercase().as_str() {
        "csv" => exportar_csv(&boveda.entradas),
        "json" => match exportar_json_plano(&boveda) {
            Ok(j) => j,
            Err(e) => {
                eprintln!("Error al exportar JSON: {}", e);
                return;
            }
        },
        otro => {
            eprintln!("Formato de exportación '{}' no soportado. Usa 'csv' o 'json'.", otro);
            return;
        }
    };

    match salida_opt {
        Some(ruta_salida) => {
            if let Err(e) = fs::write(ruta_salida, contenido_exportado) {
                eprintln!("Error al guardar archivo en '{}': {}", ruta_salida, e);
                return;
            }
            println!("✅ Exportación completada: {} entradas guardadas en '{}'.", boveda.entradas.len(), ruta_salida);
        }
        None => {
            println!("{}", contenido_exportado);
        }
    }
}

fn crear_boveda_demo(ruta: &str, password_opt: Option<&str>) {
    let password = match password_opt {
        Some(p) => p.to_string(),
        None => match leer_contrasena("🔑 Elige una contraseña maestra para la nueva bóveda: ") {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error al leer contraseña: {}", e);
                return;
            }
        },
    };

    if password.trim().is_empty() {
        eprintln!("La contraseña no puede estar vacía.");
        return;
    }

    let mut boveda = ContenidoBoveda::default();

    // Categorías de ejemplo
    boveda.categorias.push(Categoria {
        id: "cat-servicios-web".into(),
        nombre: "Servicios Web".into(),
        icono: "globe".into(),
        color_hex: Some("#2563EB".into()),
        color: None,
        creada_en: 1700000000000,
        modificada_en: 1700000000000,
    });
    boveda.categorias.push(Categoria {
        id: "cat-finanzas".into(),
        nombre: "Finanzas y Banca".into(),
        icono: "bank".into(),
        color_hex: Some("#16A34A".into()),
        color: None,
        creada_en: 1700000000000,
        modificada_en: 1700000000000,
    });

    // Perfiles de ejemplo
    boveda.perfiles.push(Perfil {
        id: "perfil-personal".into(),
        nombre: "Personal".into(),
        correo_principal: "usuario@ejemplo.com".into(),
        correos_secundarios: vec!["alias@ejemplo.com".into()],
        tipo_documento: Some("DNI".into()),
        numero_documento: Some("12345678Z".into()),
        preferencia_identificador: "correo".into(),
        color_hex: Some("#8B5CF6".into()),
        color: None,
        icono: "person".into(),
        tarjetas_ids: vec![],
        tarjeta_predeterminada_id: None,
        cuentas_ids: vec![],
        cuenta_predeterminada_id: None,
        contactos_ids: vec![],
        contacto_predeterminado_id: None,
        documentos_ids: vec![],
        documento_predeterminado_id: None,
        creada_en: 1700000000000,
        modificada_en: 1700000000000,
    });

    // Entradas demostrativas con los tipos nativos
    boveda.entradas.push(Entrada {
        id: "00000000-0000-0000-0000-000000000001".into(),
        tipo: TipoEntrada::Login,
        titulo: "Google Workspace".into(),
        usuario: "admin@empresa.com".into(),
        contrasena: "C0ntr4s3n4_Fuerte!2026".into(),
        urls: vec!["https://accounts.google.com".into()],
        notas: "Cuenta principal de trabajo.".into(),
        secreto_totp: Some("JBSWY3DPEHPK3PXP".into()),
        categorias: vec!["cat-servicios-web".into()],
        perfil_id: Some("perfil-personal".into()),
        favorito: true,
        ..Default::default()
    });

    boveda.entradas.push(Entrada {
        id: "00000000-0000-0000-0000-000000000002".into(),
        tipo: TipoEntrada::Tarjeta,
        titulo: "Tarjeta Débito Santander".into(),
        datos_tarjeta: Some(DatosTarjetaBancaria {
            titular: "Jose Luis Navas".into(),
            numero: "4532123456789012".into(),
            vencimiento: "12/29".into(),
            cvv: "789".into(),
            pin: "1234".into(),
            tipo: "Débito".into(),
            marca: "Visa".into(),
        }),
        categorias: vec!["cat-finanzas".into()],
        perfil_id: Some("perfil-personal".into()),
        favorito: true,
        ..Default::default()
    });

    boveda.entradas.push(Entrada {
        id: "00000000-0000-0000-0000-000000000003".into(),
        tipo: TipoEntrada::CuentaBancaria,
        titulo: "Cuenta Corriente Santander".into(),
        datos_cuenta_bancaria: Some(DatosCuentaBancaria {
            banco: "Banco Santander".into(),
            titular: "Jose Luis Navas".into(),
            tipo_cuenta: "Nómina".into(),
            numero_cuenta: "ES9121000418450200051332".into(),
            swift_bic: "BSCHESMMXXX".into(),
            cbu_clabe: "".into(),
        }),
        categorias: vec!["cat-finanzas".into()],
        ..Default::default()
    });

    boveda.entradas.push(Entrada {
        id: "00000000-0000-0000-0000-000000000004".into(),
        tipo: TipoEntrada::Wifi,
        titulo: "Wi-Fi Oficina".into(),
        datos_wifi: Some(DatosRedWifi {
            ssid: "FibraOptica_5G".into(),
            contrasena: "ClaveWifiUltrasegura!".into(),
            tipo_seguridad: "WPA3".into(),
            oculta: false,
        }),
        ..Default::default()
    });

    boveda.entradas.push(Entrada {
        id: "00000000-0000-0000-0000-000000000005".into(),
        tipo: TipoEntrada::Servidor,
        titulo: "Servidor Proxmox HomeLab".into(),
        datos_servidor: Some(DatosServidorSsh {
            host: "192.168.1.100".into(),
            puerto: "22".into(),
            usuario: "root".into(),
            clave_privada: "-----BEGIN OPENSSH PRIVATE KEY-----\n...".into(),
            clave_publica: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIG...".into(),
        }),
        urls: vec!["https://192.168.1.100:8006".into()],
        ..Default::default()
    });

    boveda.entradas.push(Entrada {
        id: "00000000-0000-0000-0000-000000000006".into(),
        tipo: TipoEntrada::Identidad,
        titulo: "DNI Español".into(),
        datos_identidad: Some(DatosDocumentoIdentidad {
            tipo_documento: "DNI".into(),
            numero: "12345678Z".into(),
            nombre_completo: "Jose Luis Navas".into(),
            expedicion: "2020-01-15".into(),
            caducidad: "2030-01-15".into(),
            pais_emisor: "España".into(),
        }),
        perfil_id: Some("perfil-personal".into()),
        ..Default::default()
    });

    boveda.entradas.push(Entrada {
        id: "00000000-0000-0000-0000-000000000007".into(),
        tipo: TipoEntrada::Wallet,
        titulo: "Cold Storage Bitcoin".into(),
        datos_wallet: Some(DatosCriptoWallet {
            red: "Bitcoin".into(),
            direccion_publica: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh".into(),
            frase_semilla: "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".into(),
            clave_privada: "".into(),
        }),
        ..Default::default()
    });

    boveda.entradas.push(Entrada {
        id: "00000000-0000-0000-0000-000000000008".into(),
        tipo: TipoEntrada::Contacto,
        titulo: "Contacto Abogado".into(),
        datos_contacto: Some(DatosContacto {
            nombre: "Carlos Ramos".into(),
            telefono_secundario: "+34600112233".into(),
            correo: "carlos@abogados.es".into(),
            direccion: "Paseo de la Castellana 100".into(),
            codigo_postal: "28046".into(),
            ciudad: "Madrid".into(),
            provincia: "Madrid".into(),
            pais: "España".into(),
            empresa: "Bufete Legal Ramos".into(),
        }),
        ..Default::default()
    });

    boveda.entradas.push(Entrada {
        id: "00000000-0000-0000-0000-000000000009".into(),
        tipo: TipoEntrada::Nota,
        titulo: "Plan de Respaldo Soberano".into(),
        notas: "# Procedimiento de Respaldo\n\n1. Generar copia `.bvda` semanalmente.\n2. Almacenar en medio extraíble cifrado.\n3. Verificar suma de verificación.".into(),
        ..Default::default()
    });

    println!("⏳ Cifrando bóveda con Argon2id (Perfil Estándar) y AES-256-GCM...");
    let bytes_cifrados = match empaquetar_boveda(&boveda, password.as_bytes(), PerfilArgon2::Estandar) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error al empaquetar bóveda: {}", e);
            return;
        }
    };

    let destino = Path::new(ruta);
    if let Err(e) = escribir_atomico(destino, &bytes_cifrados) {
        eprintln!("Error al escribir archivo en '{}': {}", ruta, e);
        return;
    }

    println!("🎉 Bóveda demo creada exitosamente en '{}' ({} bytes)", ruta, bytes_cifrados.len());
    println!("Puedes probarla ahora con: boveda-cli unlock {}", ruta);
}

fn gestionar_configuracion(subargs: &[String]) {
    if subargs.is_empty() {
        println!("Uso de boveda-cli config:");
        println!("  boveda-cli config list");
        println!("  boveda-cli config get <clave>");
        println!("  boveda-cli config set <clave> <valor>");
        return;
    }

    let mut config = match AppConfigDesktop::cargar() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error al cargar configuración: {}", e);
            return;
        }
    };

    match subargs[0].as_str() {
        "list" => {
            println!("⚙️ Configuración Local de Bóveda Local Desktop:");
            println!("   Ubicación: {}", AppConfigDesktop::ruta_archivo().display());
            println!("--------------------------------------------------");
            for (clave, valor) in config.listar_valores() {
                println!("  {:<32}: {}", clave, valor);
            }
            println!();
        }
        "get" => {
            if subargs.len() < 2 {
                eprintln!("Uso: boveda-cli config get <clave>");
                return;
            }
            let clave = &subargs[1];
            match config.get_valor(clave) {
                Some(val) => println!("{}: {}", clave, val),
                None => eprintln!("Clave desconocida: '{}'", clave),
            }
        }
        "set" => {
            if subargs.len() < 3 {
                eprintln!("Uso: boveda-cli config set <clave> <valor>");
                return;
            }
            let clave = &subargs[1];
            let valor = &subargs[2];
            match config.set_valor(clave, valor) {
                Ok(()) => {
                    if let Err(e) = config.guardar() {
                        eprintln!("Error al guardar configuración: {}", e);
                        return;
                    }
                    println!("✅ Ajuste '{}' actualizado a '{}'", clave, valor);
                }
                Err(e) => {
                    eprintln!("❌ Error al modificar ajuste: {}", e);
                }
            }
        }
        otro => {
            eprintln!("Subcomando desconocido: '{}'", otro);
        }
    }
}

fn hex_fmt(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}
