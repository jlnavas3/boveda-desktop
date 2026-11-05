/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Módulos de estado reactivo y micro-diseño para la interfaz Slint de Bóveda Local.

pub mod biometrics;
pub mod categories;
pub mod detail;
pub mod duplicates;
pub mod edit_mapper;
pub mod emergency_kit;
pub mod filter_engine;
pub mod health;
pub mod indicator_detector;
pub mod profiles;
pub mod rules_mapper;
pub mod selection;
pub mod settings_search;
pub mod sorting;
pub mod theme_manager;
pub mod titles;
pub mod vault_data;

pub use biometrics::*;
pub use categories::{eliminar_categoria_boveda, guardar_o_crear_categoria};
pub use detail::{
    construir_detalle_entrada, eliminar_contrasena_historial, eliminar_passkey_secundaria,
    eliminar_totp_secundario, formatear_bloques_4x4, promover_passkey_secundaria,
    promover_totp_secundario, restaurar_contrasena_historial,
};
pub use duplicates::{
    contar_entradas_secundarias_duplicadas, detectar_grupos_duplicados, unificar_grupo_duplicado,
    unificar_todos_los_duplicados, FiltroDuplicados,
};
pub use edit_mapper::{edit_data_a_entrada, entrada_a_edit_data};
pub use emergency_kit::generar_kit_emergencia_texto;
pub use filter_engine::{encontrar_id_por_letra, encontrar_indice_por_letra, subtitulo_segun_tipo};
pub use health::{
    analizar_salud_boveda, contar_problemas_salud_boveda_canonico, es_contrasena_comun,
    rotar_contrasena_rapida, PestanaSalud,
};
pub use indicator_detector::{detectar_indicadores, IndicadoresEntrada};
pub use profiles::{eliminar_perfil_boveda, guardar_o_crear_perfil, listar_perfiles_items};
pub use rules_mapper::{
    agregar_puerto_a_str, agregar_subdominio_a_str, eliminar_puerto_de_str,
    eliminar_subdominio_de_str, parse_puertos_filas, parse_puertos_filas_con_ancho,
    parse_puertos_items, parse_subdominios_filas, parse_subdominios_filas_con_ancho,
    parse_subdominios_items,
};
pub use selection::{
    alternar_anclado_lote, alternar_favorito_lote, alternar_ignorar_salud_lote,
    analizar_discrepancias_unificacion, asignar_categorias_lote, procesar_papelera_lote,
    renombrar_lote, unificar_entradas_seleccionadas, EstadoSeleccionLista,
};
pub use settings_search::buscar_subseccion_ajustes;
pub use sorting::{ordenar_entradas_referencias, CriterioOrdenacion};
pub use theme_manager::{
    obtener_paleta_acento, resolver_tema, ColoresTemaResueltos, ModoTema, PaletaAcentoInfo,
    PALETAS_ACENTO_OFICIALES,
};
pub use titles::{
    aplicar_normalizacion_titulos, previsualizar_normalizacion_titulos, FormatoTitulo,
};
pub use vault_data::GuiVaultState;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use zeroize::Zeroizing;

    use crate::crypto::{desempaquetar_boveda, PerfilArgon2};
    use crate::gui::SettingsData;
    use crate::models::{Categoria, ContenidoBoveda, Entrada, TipoEntrada};

    #[test]
    fn test_gui_state_filtrado_y_busqueda() {
        let mut boveda = ContenidoBoveda::default();
        boveda.entradas.push(Entrada {
            id: "1".to_string(),
            titulo: "GitHub Personal".to_string(),
            usuario: "jln".to_string(),
            tipo: TipoEntrada::Login,
            ..Default::default()
        });
        boveda.entradas.push(Entrada {
            id: "2".to_string(),
            titulo: "Tarjeta Débito BBVA".to_string(),
            tipo: TipoEntrada::Tarjeta,
            ..Default::default()
        });

        let mut state = GuiVaultState::new(PathBuf::from("/tmp/test.bvda"));
        state.cargar_boveda(boveda, PathBuf::from("/tmp/test.bvda"), None, None);

        assert_eq!(state.obtener_entradas_filtradas().len(), 2);

        // Búsqueda por término
        state.termino_busqueda = "github".to_string();
        let filtradas = state.obtener_entradas_filtradas();
        assert_eq!(filtradas.len(), 1);
        assert_eq!(filtradas[0].titulo.as_str(), "GitHub Personal");

        // Salto Niagara por letra
        let id_letra = state.encontrar_id_por_letra('G');
        assert_eq!(id_letra.as_deref(), Some("1"));

        // Filtro por tipo
        state.termino_busqueda.clear();
        state.filtro_tipo = "TARJETA".to_string();
        let filtradas_tipo = state.obtener_entradas_filtradas();
        assert_eq!(filtradas_tipo.len(), 1);
        assert_eq!(filtradas_tipo[0].tipo.as_str(), "TARJETA");

        // Filtro por perfil
        state.filtro_tipo = "TODOS".to_string();
        state.filtro_perfil = "perfil-inexistente".to_string();
        assert_eq!(state.obtener_entradas_filtradas().len(), 0);
        state.filtro_perfil = "TODOS".to_string();
        assert_eq!(state.obtener_entradas_filtradas().len(), 2);

        // Filtro por etiqueta
        state.filtro_etiqueta = "tag-inexistente".to_string();
        assert_eq!(state.obtener_entradas_filtradas().len(), 0);
        state.filtro_etiqueta = "TODAS".to_string();
        assert_eq!(state.obtener_entradas_filtradas().len(), 2);

        // Bloquear
        state.bloquear();
        assert!(!state.esta_desbloqueada());
        assert_eq!(state.obtener_entradas_filtradas().len(), 0);
    }

    #[test]
    fn test_gui_state_crud_y_persistencia() {
        let temp_dir = std::env::temp_dir().join(format!("bvda-crud-state-{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);
        let ruta_archivo = temp_dir.join("crud_state_test.bvda");

        let boveda = ContenidoBoveda::default();
        let clave = Zeroizing::new("PasswordSegura2026!".to_string());

        let mut state = GuiVaultState::new(ruta_archivo.clone());
        state.cargar_boveda(
            boveda,
            ruta_archivo.clone(),
            Some(clave.clone()),
            Some(PerfilArgon2::Estandar),
        );

        // 1. Crear nueva entrada
        let nueva = Entrada {
            titulo: "Mi Servidor Proxmox".to_string(),
            tipo: TipoEntrada::Servidor,
            usuario: "root".to_string(),
            ..Default::default()
        };
        let id_creada = state.guardar_o_crear_entrada(nueva).unwrap();
        assert!(!id_creada.is_empty());
        assert!(ruta_archivo.exists());

        // Verificar que persiste en disco descifrando directamente el archivo
        let bytes_disco = fs::read(&ruta_archivo).unwrap();
        let boveda_disco = desempaquetar_boveda(&bytes_disco, clave.as_bytes()).unwrap();
        assert_eq!(boveda_disco.entradas.len(), 1);
        assert_eq!(boveda_disco.entradas[0].titulo, "Mi Servidor Proxmox");

        // 2. Mover a papelera
        state.mover_a_papelera(&id_creada).unwrap();
        state.filtro_tipo = "TODOS".to_string();
        assert_eq!(state.obtener_entradas_filtradas().len(), 0);

        state.filtro_tipo = "PAPELERA".to_string();
        assert_eq!(state.obtener_entradas_filtradas().len(), 1);

        // 3. Restaurar de papelera
        state.restaurar_de_papelera(&id_creada).unwrap();
        state.filtro_tipo = "TODOS".to_string();
        assert_eq!(state.obtener_entradas_filtradas().len(), 1);

        // 4. Cambiar clave maestra
        state
            .cambiar_clave_maestra(
                "PasswordSegura2026!",
                "NuevaClaveUltra2027!",
                PerfilArgon2::Estandar,
            )
            .unwrap();

        // Verificar que abre con la nueva y falla con la vieja
        let bytes_actualizados = fs::read(&ruta_archivo).unwrap();
        assert!(desempaquetar_boveda(&bytes_actualizados, b"PasswordSegura2026!").is_err());
        assert!(desempaquetar_boveda(&bytes_actualizados, b"NuevaClaveUltra2027!").is_ok());

        // 5. Eliminar definitivamente
        state.eliminar_definitivamente(&id_creada).unwrap();
        assert_eq!(state.obtener_entradas_filtradas().len(), 0);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_gui_state_resumenes() {
        let mut boveda = ContenidoBoveda::default();
        boveda.categorias.push(Categoria {
            id: "cat-1".to_string(),
            nombre: "Trabajo".to_string(),
            icono: "💼".to_string(),
            ..Default::default()
        });
        boveda.entradas.push(Entrada {
            id: "1".to_string(),
            titulo: "Cuenta AWS".to_string(),
            tipo: TipoEntrada::Servidor,
            categorias: vec!["cat-1".to_string()],
            ..Default::default()
        });

        let mut state = GuiVaultState::new(PathBuf::from("/tmp/test.bvda"));
        state.cargar_boveda(boveda, PathBuf::from("/tmp/test.bvda"), None, None);

        let resumen_cats = state.obtener_resumen_categorias();
        assert_eq!(resumen_cats.len(), 1);
        assert_eq!(resumen_cats[0].nombre.as_str(), "Trabajo");
        assert_eq!(resumen_cats[0].conteo, 1);

        let resumen_tipos = state.obtener_resumen_tipos();
        let item_servidor = resumen_tipos.iter().find(|i| i.id.as_str() == "SERVIDOR").unwrap();
        assert_eq!(item_servidor.conteo, 1);
        let item_papelera = resumen_tipos.iter().find(|i| i.id.as_str() == "PAPELERA").unwrap();
        assert_eq!(item_papelera.conteo, 0);
    }

    #[test]
    fn test_conversion_edit_data() {
        let entrada = Entrada {
            id: "test-id-1".into(),
            tipo: TipoEntrada::Login,
            titulo: "GitHub Corporativo".into(),
            usuario: "ingenieria@empresa.com".into(),
            favorito: true,
            notas: "Acceso con 2FA y llave YubiKey".into(),
            secreto_totp: Some("JBSWY3DPEHPK3PXP".into()),
            totp_emisor: "GitHub".into(),
            fecha_caducidad: Some(1830297599000),
            etiquetas: vec!["dev".into(), "infra".into()],
            urls: vec!["https://github.com".into(), "https://gist.github.com".into()],
            campos_personalizados: vec![crate::models::campo_personalizado::CampoPersonalizado {
                id: "cp-1".into(),
                etiqueta: "Organizacion".into(),
                valor: "BovedaCorp".into(),
                tipo: crate::models::tipo_campo::TipoCampo::Texto,
                es_sensible: false,
                es_obligatorio: false,
                formato: None,
            }],
            ..Default::default()
        };

        let edit_data = entrada_a_edit_data(&entrada);
        assert_eq!(edit_data.id.as_str(), "test-id-1");
        assert_eq!(edit_data.tipo.as_str(), "LOGIN");
        assert_eq!(edit_data.titulo.as_str(), "GitHub Corporativo");
        assert_eq!(edit_data.totp_secreto.as_str(), "JBSWY3DPEHPK3PXP");
        assert_eq!(edit_data.totp_emisor.as_str(), "GitHub");
        assert_eq!(edit_data.fecha_caducidad.as_str(), "2027-12-31");
        assert_eq!(edit_data.etiquetas.as_str(), "dev, infra");
        assert_eq!(edit_data.urls_secundarias.as_str(), "https://gist.github.com");
        assert!(edit_data.campos_personalizados.as_str().contains("Organizacion: BovedaCorp"));

        let mut edit_modificada = edit_data;
        edit_modificada.titulo = "GitHub Enterprise".into();
        edit_modificada.totp_secreto = "HXDMVJECJJWSRB3HWIZR4IFUGFTMXBOZ".into();
        edit_modificada.etiquetas = "dev, infra, cloud".into();
        edit_modificada.urls_secundarias = "https://gist.github.com\nhttps://cli.github.com".into();
        edit_modificada.campos_personalizados = "Organizacion: BovedaCorp\nDepartamento: Seguridad".into();

        let entrada_reconvertida = edit_data_a_entrada(&edit_modificada, Some(&entrada));
        assert_eq!(entrada_reconvertida.id, "test-id-1");
        assert_eq!(entrada_reconvertida.titulo, "GitHub Enterprise");
        assert_eq!(entrada_reconvertida.secreto_totp.as_deref(), Some("HXDMVJECJJWSRB3HWIZR4IFUGFTMXBOZ"));
        assert_eq!(entrada_reconvertida.etiquetas, vec!["dev", "infra", "cloud"]);
        assert_eq!(entrada_reconvertida.urls.len(), 3);
        assert_eq!(entrada_reconvertida.urls[0], "https://github.com");
        assert_eq!(entrada_reconvertida.urls[1], "https://gist.github.com");
        assert_eq!(entrada_reconvertida.urls[2], "https://cli.github.com");
        let cp_depto = entrada_reconvertida.campos_personalizados.iter().find(|c| c.etiqueta == "Departamento");
        assert_eq!(cp_depto.map(|c| c.valor.as_str()), Some("Seguridad"));
    }

    #[test]
    fn test_gui_state_actualizar_ajustes() {
        let temp_dir = std::env::temp_dir().join(format!("bvda-settings-state-{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);
        let ruta_archivo = temp_dir.join("ajustes_state_test.bvda");

        let boveda = ContenidoBoveda::default();
        let clave = Zeroizing::new("MasterKey2026!".to_string());

        let mut state = GuiVaultState::new(ruta_archivo.clone());
        state.cargar_boveda(
            boveda,
            ruta_archivo.clone(),
            Some(clave.clone()),
            Some(PerfilArgon2::Estandar),
        );

        let datos_ajustes = SettingsData {
            portapapeles_segundos: 45,
            autobloqueo_segundos: 180,
            perfil_argon2: "REFORZADO".into(),
            ocultar_usuario: true,
            ocultar_contrasena: true,
            ocultar_notas: true,
            ocultar_campos: true,
            criterio_ordenacion: "TITULO".into(),
            fijar_anclados_arriba: true,
            prefijos_subdominios: "dev, test, lab".into(),
            puertos_homelab: "8006: Proxmox\n8443: UniFi".into(),
            ..Default::default()
        };

        state.actualizar_ajustes(&datos_ajustes, PerfilArgon2::Reforzado).unwrap();
        assert_eq!(state.perfil_argon2, PerfilArgon2::Reforzado);

        let bytes_disco = fs::read(&ruta_archivo).unwrap();
        let boveda_disco = desempaquetar_boveda(&bytes_disco, clave.as_bytes()).unwrap();
        let config = boveda_disco.configuracion.unwrap();
        assert_eq!(config.portapapeles_segundos, 45);
        assert_eq!(config.auto_bloqueo_segundos, 180);
        assert!(config.ocultar_usuario);
        assert_eq!(config.prefijos_subdominios, vec!["dev", "test", "lab"]);
        assert_eq!(config.puertos_servicios_locales.get("8006").map(|s| s.as_str()), Some("Proxmox"));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_encontrar_indice_por_letra_y_flags() {
        use crate::gui::EntryOverview;
        use crate::gui::state::encontrar_indice_por_letra;

        let entradas = vec![
            EntryOverview {
                id: "1".into(),
                titulo: "Apple iCloud".into(),
                subtitulo: "user@apple.com".into(),
                tipo: "LOGIN".into(),
                categoria_id: "general".into(),
                tiene_usuario: true,
                tiene_password: true,
                tiene_url: true,
                servicio: "Apple".into(),
                ..Default::default()
            },
            EntryOverview {
                id: "2".into(),
                titulo: "GitHub".into(),
                subtitulo: "octocat".into(),
                tipo: "LOGIN".into(),
                favorito: true,
                categoria_id: "dev".into(),
                tiene_usuario: true,
                tiene_password: true,
                tiene_totp: true,
                tiene_url: true,
                tiene_notas: true,
                totp_codigo: "123 456".into(),
                totp_restante: 18,
                servicio: "Github".into(),
                ..Default::default()
            },
            EntryOverview {
                id: "3".into(),
                titulo: "Google Workspace".into(),
                subtitulo: "admin@corp.com".into(),
                tipo: "LOGIN".into(),
                categoria_id: "trabajo".into(),
                tiene_usuario: true,
                tiene_totp: true,
                tiene_url: true,
                totp_codigo: "789 012".into(),
                totp_restante: 25,
                servicio: "Google".into(),
                ..Default::default()
            },
        ];

        assert_eq!(encontrar_indice_por_letra(&entradas, 'A'), Some(0));
        assert_eq!(encontrar_indice_por_letra(&entradas, 'a'), Some(0));
        assert_eq!(encontrar_indice_por_letra(&entradas, 'G'), Some(1));
        assert_eq!(encontrar_indice_por_letra(&entradas, 'Z'), None);
    }

    #[test]
    fn test_fases_1_2_3_paridad_android() {
        use crate::models::cambio_contrasena::CambioContrasena;
        use crate::models::passkey::DatosPasskey;
        use crate::models::totp_secundario::TotpSecundario;

        // Fase 1: Credenciales alternativas y Llave de acceso
        let mut e1 = Entrada {
            id: "id-1".to_string(),
            titulo: "GitHub".to_string(),
            usuario: "jln".to_string(),
            contrasena: "PassActual!".to_string(),
            historial_contrasenas: vec![CambioContrasena {
                contrasena: "PassAntigua!".to_string(),
                cambiada_en: 1700000000000,
            }],
            passkey: Some(DatosPasskey {
                cred_id: "cred-main".to_string(),
                rp_id: "github.com".to_string(),
                rp_name: "GitHub".to_string(),
                user_handle: "uh-1".to_string(),
                algoritmo: "ES256".to_string(),
                creada_en: 1700000000000,
                usuario: "jln".to_string(),
                clave_privada: "pk1".to_string(),
            }),
            passkeys_secundarias: vec![DatosPasskey {
                cred_id: "cred-sec".to_string(),
                rp_id: "github.com".to_string(),
                rp_name: "GitHub Alt".to_string(),
                user_handle: "uh-2".to_string(),
                algoritmo: "ES256".to_string(),
                creada_en: 1700000000000,
                usuario: "jln-alt".to_string(),
                clave_privada: "pk2".to_string(),
            }],
            secreto_totp: Some("JBSWY3DPEHPK3PXP".to_string()),
            totps_secundarios: vec![TotpSecundario {
                id: "totp-sec-1".to_string(),
                secreto: "HXDMVJECJJWSRB3HWIZR4IFUGFTMXBOZ".to_string(),
                emisor: "GitHub Sec".to_string(),
                digitos: 6,
                periodo: 30,
                algoritmo: "HmacSHA1".to_string(),
                agregado_en: 1700000000000,
            }],
            ..Default::default()
        };

        assert!(restaurar_contrasena_historial(&mut e1, "PassAntigua!"));
        assert_eq!(e1.contrasena, "PassAntigua!");
        assert_eq!(e1.historial_contrasenas[0].contrasena, "PassActual!");

        assert!(promover_passkey_secundaria(&mut e1, "cred-sec"));
        assert_eq!(e1.passkey.as_ref().unwrap().cred_id, "cred-sec");
        assert_eq!(e1.passkeys_secundarias[0].cred_id, "cred-main");

        assert!(promover_totp_secundario(&mut e1, "totp-sec-1"));
        assert_eq!(e1.secreto_totp.as_deref(), Some("HXDMVJECJJWSRB3HWIZR4IFUGFTMXBOZ"));

        // Fase 2: Selección múltiple y unificación sin pérdida
        let e2 = Entrada {
            id: "id-2".to_string(),
            titulo: "GitHub Secondary".to_string(),
            usuario: "jln2".to_string(),
            contrasena: "PassSegundaCuenta!".to_string(),
            ..Default::default()
        };
        let mut boveda = ContenidoBoveda {
            entradas: vec![e1, e2],
            ..Default::default()
        };
        let mut sel = EstadoSeleccionLista::new();
        sel.alternar_id("id-1");
        sel.alternar_id("id-2");
        assert_eq!(sel.conteo(), 2);

        let id_unificado = unificar_entradas_seleccionadas(
            &mut boveda,
            sel.ids_seleccionados(),
            Some("id-1"),
        );
        assert_eq!(id_unificado.as_deref(), Some("id-1"));
        assert_eq!(boveda.entradas.len(), 1);
        assert!(boveda.entradas[0]
            .historial_contrasenas
            .iter()
            .any(|h| h.contrasena == "PassSegundaCuenta!"));

        // Fase 3: Cambio de TipoEntrada y respaldo automático de contraseña al editar
        let mut edit = entrada_a_edit_data(&boveda.entradas[0]);
        edit.tipo = "SERVIDOR".into();
        edit.password = "NuevaPassword2026!".into();
        edit.servidor_host = "10.0.0.5".into();
        edit.servidor_clave_publica = "ssh-ed25519 AAAAC3Nza...".into();
        let actualizada = edit_data_a_entrada(&edit, Some(&boveda.entradas[0]));
        assert_eq!(actualizada.tipo, TipoEntrada::Servidor);
        assert_eq!(
            actualizada.datos_servidor.as_ref().unwrap().clave_publica,
            "ssh-ed25519 AAAAC3Nza..."
        );
        assert!(actualizada
            .historial_contrasenas
            .iter()
            .any(|h| h.contrasena == "PassAntigua!"));
    }

    #[test]
    fn test_fases_1_a_6_salud_duplicados_titulos_crud_ordenacion() {
        use crate::gui::state::categories::{eliminar_categoria_boveda, guardar_o_crear_categoria};
        use crate::gui::state::detail::formatear_bloques_4x4;
        use crate::gui::state::duplicates::{
            detectar_grupos_duplicados, unificar_todos_los_duplicados,
        };
        use crate::gui::state::health::{analizar_salud_boveda, rotar_contrasena_rapida};
        use crate::gui::state::profiles::{eliminar_perfil_boveda, guardar_o_crear_perfil};
        use crate::gui::state::sorting::ordenar_entradas_referencias;
        use crate::gui::state::titles::{
            aplicar_normalizacion_titulos, previsualizar_normalizacion_titulos,
        };
        use crate::gui::state::buscar_subseccion_ajustes;

        // 1. Ordenación
        let mut e1 = Entrada {
            id: "e1".into(),
            titulo: "github.com".into(),
            usuario: "dev@corp.com".into(),
            contrasena: "password".into(),
            urls: vec!["https://github.com/login".into()],
            creada_en: 1000,
            modificada_en: 5000,
            ultimo_uso_en: 10000,
            ..Default::default()
        };
        let e2 = Entrada {
            id: "e2".into(),
            titulo: "github.com".into(),
            usuario: "dev@corp.com".into(),
            contrasena: "password".into(),
            urls: vec!["https://github.com/settings".into()],
            creada_en: 2000,
            modificada_en: 3000,
            ultimo_uso_en: 2000,
            ..Default::default()
        };
        let mut refs = vec![&e1, &e2];
        ordenar_entradas_referencias(&mut refs, "FRECUENCIA_USO", false);
        assert_eq!(refs[0].id, "e1");

        // 2. Salud de la bóveda y rotación rápida
        let boveda_salud = ContenidoBoveda {
            entradas: vec![e1.clone(), e2.clone()],
            ..Default::default()
        };
        let (puntaje, conteos, items_comunes) =
            analizar_salud_boveda(&boveda_salud, "COMUNES");
        assert!(puntaje < 100);
        assert_eq!(conteos[0], 0); // Caducadas
        assert_eq!(conteos[1], 2); // Repetidas
        assert_eq!(conteos[2], 2); // Comunes
        assert_eq!(conteos[3], 2); // Débiles
        assert_eq!(items_comunes.len(), 2);

        assert!(rotar_contrasena_rapida(&mut e1, "NuevaClaveFuerte2026!#"));
        assert_eq!(e1.contrasena, "NuevaClaveFuerte2026!#");
        assert_eq!(e1.historial_contrasenas[0].contrasena, "password");

        // 3. Duplicados y unificación masiva
        let mut boveda = ContenidoBoveda {
            entradas: vec![e1, e2],
            ..Default::default()
        };
        let grupos = detectar_grupos_duplicados(&boveda, "TODOS");
        assert_eq!(grupos.len(), 1);
        let unificados = unificar_todos_los_duplicados(&mut boveda, "TODOS");
        assert_eq!(unificados, 1);
        assert_eq!(boveda.entradas.len(), 1);

        // 4. Normalizador de títulos
        let preview = previsualizar_normalizacion_titulos(
            &boveda,
            "CAPITALIZADO",
            false,
        );
        assert_eq!(preview.len(), 1);
        let modificadas = aplicar_normalizacion_titulos(
            &mut boveda,
            "CAPITALIZADO",
            false,
            None,
            None,
        );
        assert_eq!(modificadas, 1);
        assert_eq!(boveda.entradas[0].titulo, "GitHub");

        // 5. CRUD Categorías y Perfiles
        let cat_id = guardar_o_crear_categoria(&mut boveda, "", "Finanzas", "💰").unwrap();
        boveda.entradas[0].categorias.push(cat_id.clone());
        assert!(eliminar_categoria_boveda(&mut boveda, &cat_id));
        assert!(boveda.entradas[0].categorias.is_empty());

        let prof_id = guardar_o_crear_perfil(&mut boveda, "", "Corporativo", "corp@test.com", "", "CI", "1700000001", "correo").unwrap();
        boveda.entradas[0].perfil_id = Some(prof_id.clone());
        assert!(eliminar_perfil_boveda(&mut boveda, &prof_id));
        assert_eq!(boveda.entradas[0].perfil_id, None);

        // 6. Buscador de ajustes y formato 4x4
        assert_eq!(buscar_subseccion_ajustes("argon2"), Some((1, 2)));
        assert_eq!(buscar_subseccion_ajustes("kit de emergencia"), Some((5, 404)));
        assert_eq!(
            formatear_bloques_4x4("4111222233334444", true),
            "4111 2222 3333 4444"
        );
    }
}
