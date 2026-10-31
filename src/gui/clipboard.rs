use std::thread;
use std::time::Duration;
use zeroize::Zeroize;

/// Gestor soberano de portapapeles con limpieza temporizada automática de RAM y portapapeles.
pub struct ClipboardManager;

impl ClipboardManager {
    /// Copia un texto sensible al portapapeles y programa su borrado automático tras `segundos`.
    pub fn copiar_con_limpieza(texto: String, segundos: u32) -> Result<(), String> {
        let mut clipboard = arboard::Clipboard::new()
            .map_err(|e| format!("Error al acceder al portapapeles del sistema: {e}"))?;

        clipboard
            .set_text(texto.clone())
            .map_err(|e| format!("Error al escribir en el portapapeles: {e}"))?;

        if segundos > 0 {
            // Lanzamos hilo desacoplado para limpiar el portapapeles si el contenido no ha cambiado
            thread::spawn(move || {
                thread::sleep(Duration::from_secs(segundos as u64));

                if let Ok(mut cb) = arboard::Clipboard::new() {
                    if let Ok(actual) = cb.get_text() {
                        if actual == texto {
                            let _ = cb.set_text(String::new());
                        }
                    }
                }
                let mut tmp = texto;
                tmp.zeroize();
            });
        }

        Ok(())
    }
}
