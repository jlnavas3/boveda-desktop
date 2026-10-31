/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Módulos de estado reactivo y micro-diseño para la interfaz Slint de Bóveda Local.

pub mod biometrics;
pub mod edit_mapper;
pub mod filter_engine;
pub mod indicator_detector;
pub mod rules_mapper;
pub mod theme_manager;
pub mod vault_data;

pub use biometrics::*;
pub use edit_mapper::{edit_data_a_entrada, entrada_a_edit_data};
pub use filter_engine::{encontrar_id_por_letra, encontrar_indice_por_letra, subtitulo_segun_tipo};
pub use indicator_detector::{detectar_indicadores, IndicadoresEntrada};
pub use rules_mapper::{
    agregar_puerto_a_str, agregar_subdominio_a_str, eliminar_puerto_de_str,
    eliminar_subdominio_de_str, parse_puertos_filas, parse_puertos_filas_con_ancho,
    parse_puertos_items, parse_subdominios_filas, parse_subdominios_filas_con_ancho,
    parse_subdominios_items,
};
pub use theme_manager::{
    obtener_paleta_acento, resolver_tema, ColoresTemaResueltos, ModoTema, PaletaAcentoInfo,
    PALETAS_ACENTO_OFICIALES,
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
                favorito: false,
                anclado: false,
                categoria_id: "general".into(),
                tiene_usuario: true,
                tiene_password: true,
                tiene_totp: false,
                tiene_passkey: false,
                tiene_url: true,
                tiene_app: false,
                tiene_notas: false,
                totp_codigo: "".into(),
                totp_restante: 0,
                servicio: "Apple".into(),
                es_cabecera_grupo: false,
                clave_grupo: "".into(),
                grupo_conteo: 0,
                grupo_expandido: false,
                es_hijo_grupo: false,
                es_ultimo_hijo_grupo: false,
            },
            EntryOverview {
                id: "2".into(),
                titulo: "GitHub".into(),
                subtitulo: "octocat".into(),
                tipo: "LOGIN".into(),
                favorito: true,
                anclado: false,
                categoria_id: "dev".into(),
                tiene_usuario: true,
                tiene_password: true,
                tiene_totp: true,
                tiene_passkey: false,
                tiene_url: true,
                tiene_app: false,
                tiene_notas: true,
                totp_codigo: "123 456".into(),
                totp_restante: 18,
                servicio: "Github".into(),
                es_cabecera_grupo: false,
                clave_grupo: "".into(),
                grupo_conteo: 0,
                grupo_expandido: false,
                es_hijo_grupo: false,
                es_ultimo_hijo_grupo: false,
            },
            EntryOverview {
                id: "3".into(),
                titulo: "Google Workspace".into(),
                subtitulo: "admin@corp.com".into(),
                tipo: "LOGIN".into(),
                favorito: false,
                anclado: false,
                categoria_id: "trabajo".into(),
                tiene_usuario: true,
                tiene_password: false,
                tiene_totp: true,
                tiene_passkey: false,
                tiene_url: true,
                tiene_app: false,
                tiene_notas: false,
                totp_codigo: "789 012".into(),
                totp_restante: 25,
                servicio: "Google".into(),
                es_cabecera_grupo: false,
                clave_grupo: "".into(),
                grupo_conteo: 0,
                grupo_expandido: false,
                es_hijo_grupo: false,
                es_ultimo_hijo_grupo: false,
            },
        ];

        assert_eq!(encontrar_indice_por_letra(&entradas, 'A'), Some(0));
        assert_eq!(encontrar_indice_por_letra(&entradas, 'a'), Some(0));
        assert_eq!(encontrar_indice_por_letra(&entradas, 'G'), Some(1));
        assert_eq!(encontrar_indice_por_letra(&entradas, 'Z'), None);
    }
}
