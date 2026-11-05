/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Servidor TCP efímero para el flujo de sincronización LAN P2P y Atomic Commit.
//! Escucha conexiones entrantes, ejecuta autenticación mutua 1-RTT, fusiona bóvedas
//! bidireccionalmente y persiste el resultado de forma atómica.

use std::net::{TcpListener, TcpStream};
use std::time::{Duration, Instant};

use crate::models::ContenidoBoveda;
use crate::sync::merge::vault_merge::fusionar_bovedas_con_informe;
use crate::sync::merge::sync_report::InformeSincronizacion;
use crate::sync::p2p::handshake::responder_server_hello;
use crate::sync::p2p::session_crypto::SessionCipher;
use crate::sync::p2p::wire_protocol::{
    deserializar_mensaje, escribir_trama, leer_trama, serializar_mensaje, MensajeP2P,
    MAX_TRAMA_TAMANO_DEFECTO,
};

/// Configuración operativa del servidor TCP efímero de sincronización.
#[derive(Clone)]
pub struct ConfiguracionServidorP2P {
    /// Clave privada escalar X25519 del Desktop (32 bytes).
    pub sk_desktop: [u8; 32],
    /// Token secreto precompartido del código QR (32 bytes).
    pub psk_token: [u8; 32],
    /// Puerto de escucha (0 para asignación efímera dinámica del sistema operativo).
    pub puerto: u16,
    /// Timeout global de espera y transferencia (por defecto 180 segundos).
    pub timeout_segundos: u64,
}

impl ConfiguracionServidorP2P {
    pub fn new(sk_desktop: [u8; 32], psk_token: [u8; 32], puerto: u16) -> Self {
        Self {
            sk_desktop,
            psk_token,
            puerto,
            timeout_segundos: 180,
        }
    }
}

/// Resultado de una sesión de sincronización P2P exitosa.
#[derive(Debug, Clone)]
pub struct ResultadoSyncServidor {
    /// Bóveda fusionada determinista y sin pérdida de datos.
    pub boveda_fusionada: ContenidoBoveda,
    /// Mensaje de confirmación ACK recibido del cliente remoto.
    pub ack_recibido: String,
    /// Clave pública del cliente remoto (Android) verificada.
    pub pk_android: [u8; 32],
    /// Informe estructurado con balance y máscaras de diff.
    pub informe_sync: InformeSincronizacion,
}

/// Servidor TCP efímero que orquesta la sincronización P2P.
pub struct ServidorSyncP2P {
    listener: TcpListener,
    config: ConfiguracionServidorP2P,
}

impl ServidorSyncP2P {
    /// Enlaza el servidor al socket `0.0.0.0:<puerto>` especificado.
    pub fn bind(config: ConfiguracionServidorP2P) -> Result<Self, String> {
        let direccion = format!("0.0.0.0:{}", config.puerto);
        let listener = TcpListener::bind(&direccion)
            .map_err(|e| format!("Error enlazando servidor TCP en {direccion}: {e}"))?;

        Ok(Self { listener, config })
    }

    /// Retorna el puerto local real asignado por el sistema operativo (esencial para puerto 0 efímero).
    pub fn puerto_local(&self) -> Result<u16, String> {
        self.listener
            .local_addr()
            .map(|a| a.port())
            .map_err(|e| format!("Error obteniendo puerto local del socket: {e}"))
    }

    /// Acepta una única conexión entrante y ejecuta el ciclo de sincronización completo:
    /// 1. Lee `ClientHello` (64 bytes).
    /// 2. Responde `ServerHello` (32 bytes) autenticado con PSK y deriva claves de sesión.
    /// 3. Recibe la bóveda remota del cliente cifrada con ChaCha20-Poly1305.
    /// 4. Ejecuta `fusionar_bovedas(local, remota)`.
    /// 5. Cifra y envía al cliente la bóveda fusionada resultante.
    /// 6. Espera la confirmación `CommitAck` del cliente.
    /// 7. Invoca `guardar_commit_local(&boveda_fusionada)` para persistencia atómica.
    /// 8. Cierra la conexión TCP limpiamente.
    pub fn atender_sincronizacion<F>(
        self,
        boveda_local: &ContenidoBoveda,
        guardar_commit_local: F,
    ) -> Result<ResultadoSyncServidor, String>
    where
        F: FnOnce(&ContenidoBoveda) -> Result<(), String>,
    {
        let dummy_cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        self.atender_sincronizacion_cancelable_con_confirmacion(
            boveda_local,
            guardar_commit_local,
            dummy_cancel,
            |_, _| true,
        )
    }

    /// Acepta una conexión entrante con soporte para cancelación reactiva instantánea.
    pub fn atender_sincronizacion_cancelable<F>(
        self,
        boveda_local: &ContenidoBoveda,
        guardar_commit_local: F,
        cancel_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Result<ResultadoSyncServidor, String>
    where
        F: FnOnce(&ContenidoBoveda) -> Result<(), String>,
    {
        self.atender_sincronizacion_cancelable_con_confirmacion(
            boveda_local,
            guardar_commit_local,
            cancel_flag,
            |_, _| true,
        )
    }

    /// Acepta una conexión entrante con soporte para cancelación y confirmación interactiva del dispositivo par.
    pub fn atender_sincronizacion_cancelable_con_confirmacion<F, C>(
        self,
        boveda_local: &ContenidoBoveda,
        guardar_commit_local: F,
        cancel_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
        confirmar_par: C,
    ) -> Result<ResultadoSyncServidor, String>
    where
        F: FnOnce(&ContenidoBoveda) -> Result<(), String>,
        C: FnOnce(&str, &str) -> bool,
    {
        let stream = self.esperar_conexion(cancel_flag)?;
        self.procesar_sesion(stream, boveda_local, guardar_commit_local, confirmar_par)
    }

    fn esperar_conexion(
        &self,
        cancel_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Result<TcpStream, String> {
        self.listener
            .set_nonblocking(true)
            .map_err(|e| format!("Error configurando listener no-bloqueante: {e}"))?;

        let timeout = Duration::from_secs(self.config.timeout_segundos);
        let inicio = Instant::now();

        loop {
            if cancel_flag.load(std::sync::atomic::Ordering::SeqCst) {
                return Err("Sincronización P2P cancelada por el usuario".to_string());
            }

            match self.listener.accept() {
                Ok((stream, _addr)) => {
                    stream
                        .set_nonblocking(false)
                        .map_err(|e| format!("Error configurando stream bloqueante: {e}"))?;
                    stream
                        .set_read_timeout(Some(timeout))
                        .map_err(|e| format!("Error configurando timeout de lectura: {e}"))?;
                    stream
                        .set_write_timeout(Some(timeout))
                        .map_err(|e| format!("Error configurando timeout de escritura: {e}"))?;
                    return Ok(stream);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    if inicio.elapsed() >= timeout {
                        return Err(format!(
                            "Tiempo de espera agotado (timeout {}s) esperando conexión P2P",
                            self.config.timeout_segundos
                        ));
                    }
                    std::thread::sleep(Duration::from_millis(30));
                }
                Err(e) => return Err(format!("Fallo aceptando conexión TCP: {e}")),
            }
        }
    }

    fn procesar_sesion<F, C>(
        &self,
        mut stream: TcpStream,
        boveda_local: &ContenidoBoveda,
        guardar_commit_local: F,
        confirmar_par: C,
    ) -> Result<ResultadoSyncServidor, String>
    where
        F: FnOnce(&ContenidoBoveda) -> Result<(), String>,
        C: FnOnce(&str, &str) -> bool,
    {
        // 1. Leer trama inicial (puede ser SaludoP2P previo o ClientHello directo)
        let primera_trama = leer_trama(&mut stream, 1024)?;
        let (client_hello_raw, alias_anunciado) = if primera_trama.len() == 64 {
            (primera_trama, None)
        } else if let Ok(saludo) =
            serde_json::from_slice::<crate::sync::p2p::device_name::SaludoP2P>(&primera_trama)
        {
            let saludo_local = crate::sync::p2p::device_name::SaludoP2P::local();
            let resp = serde_json::to_vec(&saludo_local)
                .map_err(|e| format!("Error serializando saludo local: {e}"))?;
            escribir_trama(&mut stream, &resp)?;

            let siguiente = leer_trama(&mut stream, 1024)?;
            if siguiente.len() != 64 {
                return Err(format!(
                    "Trama ClientHello inválida tras saludo: esperados 64 bytes, recibidos {}",
                    siguiente.len()
                ));
            }
            (siguiente, Some(saludo.alias_dispositivo))
        } else {
            return Err(format!(
                "Trama inicial desconocida: longitud {} bytes",
                primera_trama.len()
            ));
        };

        let mut client_hello = [0u8; 64];
        client_hello.copy_from_slice(&client_hello_raw);

        // Extraer pk_android para el resultado
        let mut pk_android = [0u8; 32];
        pk_android.copy_from_slice(&client_hello[0..32]);

        // 2. Responder ServerHello y derivar claves de sesión
        let (server_hello, keys) = responder_server_hello(
            &self.config.sk_desktop,
            &self.config.psk_token,
            &client_hello,
        )?;

        // 3. Enviar ServerHello (32 bytes)
        escribir_trama(&mut stream, &server_hello)?;

        // 4. Instanciar cifrador de sesión simétrica con Anti-Replay
        let mut cipher = SessionCipher::para_servidor(&keys);

        // 5. Recibir payload de bóveda del cliente (cifrada con ChaCha20-Poly1305)
        let frame_boveda_remota = leer_trama(&mut stream, MAX_TRAMA_TAMANO_DEFECTO)?;
        let plaintext_remoto = cipher.open(&frame_boveda_remota)?;
        let msg_remoto = deserializar_mensaje(&plaintext_remoto)?;

        let boveda_remota = match msg_remoto {
            MensajeP2P::SyncPayload(b) => *b,
            MensajeP2P::CommitAck(_) => {
                return Err("Protocolo P2P violado: se esperaba SyncPayload, se recibió CommitAck".into());
            }
        };

        // 6. Verificar y confirmar dispositivo par antes del commit
        let alias_efectivo = alias_anunciado
            .or_else(|| boveda_remota.alias_dispositivo.clone())
            .unwrap_or_else(|| "Dispositivo Móvil".to_string());

        let pk_local = crate::sync::p2p::handshake::derivar_clave_publica_x25519(&self.config.sk_desktop);
        let huella_visual = crate::sync::p2p::fingerprint::calcular_huella_visual(&pk_local, &self.config.psk_token);

        if !confirmar_par(&alias_efectivo, &huella_visual) {
            return Err("Sincronización rechazada por el usuario al verificar dispositivo par".into());
        }

        // 7. Fusión determinista y sin pérdida de datos emitiendo informe en un único pase
        let (boveda_fusionada, mut informe_sync) = fusionar_bovedas_con_informe(boveda_local, &boveda_remota);
        if informe_sync.dispositivo_remoto.is_none() && !alias_efectivo.is_empty() {
            informe_sync.dispositivo_remoto = Some(alias_efectivo);
        }

        // 8. Cifrar y enviar la bóveda fusionada al cliente con la identidad del equipo local
        let mut boveda_para_cliente = boveda_fusionada.clone();
        boveda_para_cliente.alias_dispositivo = Some(crate::sync::p2p::device_name::obtener_nombre_desktop());
        let msg_fusionada = MensajeP2P::SyncPayload(Box::new(boveda_para_cliente));
        let plaintext_fusionada = serializar_mensaje(&msg_fusionada)?;
        let frame_fusionada = cipher.seal(&plaintext_fusionada)?;
        escribir_trama(&mut stream, &frame_fusionada)?;

        // 9. Esperar confirmación ACK del cliente
        let frame_ack = leer_trama(&mut stream, 1024)?;
        let plaintext_ack = cipher.open(&frame_ack)?;
        let msg_ack = deserializar_mensaje(&plaintext_ack)?;

        let ack_recibido = match msg_ack {
            MensajeP2P::CommitAck(ack) => ack,
            MensajeP2P::SyncPayload(_) => {
                return Err("Protocolo P2P violado: se esperaba CommitAck tras envío de bóveda fusionada".into());
            }
        };

        // 10. Ejecutar commit atómico local
        guardar_commit_local(&boveda_fusionada)?;

        // 11. Cierre limpio de socket TCP
        let _ = stream.shutdown(std::net::Shutdown::Both);

        Ok(ResultadoSyncServidor {
            boveda_fusionada,
            ack_recibido,
            pk_android,
            informe_sync,
        })
    }
}
