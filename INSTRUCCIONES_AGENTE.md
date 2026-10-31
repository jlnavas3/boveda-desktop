# Instrucciones para el Agente Desarrollador (Bóveda Local Desktop)

Bienvenido, Agente. Tu misión es desarrollar la **aplicación de escritorio multiplataforma (Windows y Linux)** complementaria para el ecosistema **Bóveda Local**, utilizando **Rust** para el núcleo criptográfico, sincronización y lógica de datos, y **Slint** o **Tauri** para la interfaz gráfica de usuario.

---

## 🧭 1. Repositorio de Referencia Canónico (¡Obligatorio consultar!)

Antes de escribir código o diseñar esquemas de datos, **DEBES revisar el proyecto hermano Android de Bóveda Local** ubicado en:
`/home/jln/BovedaLocal`

En dicho directorio encontrarás la implementación de referencia probada y documentada:

* **Arquitectura y Especificación Técnica**:
  * [`/home/jln/BovedaLocal/ARQUITECTURA.md`](/home/jln/BovedaLocal/ARQUITECTURA.md) — Documento maestro con el pipeline criptográfico, cabecera de 49 bytes de `.bvda`, máquina de estados y modelos.
  * [`/home/jln/BovedaLocal/README.md`](/home/jln/BovedaLocal/README.md) — Resumen funcional de capacidades, taxonomía y directivas de seguridad.
* **Criptografía y Formato `.bvda`**:
  * [`app/src/main/java/com/jlnavas3/bovedalocal/crypto/VaultCrypto.kt`](/home/jln/BovedaLocal/app/src/main/java/com/jlnavas3/bovedalocal/crypto/VaultCrypto.kt) — Lógica de empaquetado binario, AAD de cabecera, Argon2id y AES-256-GCM.
  * [`app/src/main/java/com/jlnavas3/bovedalocal/crypto/CompresorBoveda.kt`](/home/jln/BovedaLocal/app/src/main/java/com/jlnavas3/bovedalocal/crypto/CompresorBoveda.kt) — Compresión Deflate con marca mágica `BLZ1`.
  * [`app/src/main/java/com/jlnavas3/bovedalocal/crypto/Zeroizar.kt`](/home/jln/BovedaLocal/app/src/main/java/com/jlnavas3/bovedalocal/crypto/Zeroizar.kt) — Política de sobreescritura física de memoria RAM con `0x00`.
  * [`app/src/main/java/com/jlnavas3/bovedalocal/crypto/Totp.kt`](/home/jln/BovedaLocal/app/src/main/java/com/jlnavas3/bovedalocal/crypto/Totp.kt) — Algoritmo RFC 6238 TOTP (SHA-1/256/512).
* **Modelos de Dominio y Serialización JSON**:
  * [`app/src/main/java/com/jlnavas3/bovedalocal/data/ContenidoBoveda.kt`](/home/jln/BovedaLocal/app/src/main/java/com/jlnavas3/bovedalocal/data/ContenidoBoveda.kt) — Estructura raíz serializada en el archivo `.bvda`.
  * [`app/src/main/java/com/jlnavas3/bovedalocal/data/Entrada.kt`](/home/jln/BovedaLocal/app/src/main/java/com/jlnavas3/bovedalocal/data/Entrada.kt) — Modelo inmutable de entradas.
  * [`app/src/main/java/com/jlnavas3/bovedalocal/data/TipoEntrada.kt`](/home/jln/BovedaLocal/app/src/main/java/com/jlnavas3/bovedalocal/data/TipoEntrada.kt) — Los 9 tipos de datos nativos.
  * Submodelos: `DatosTarjetaBancaria.kt`, `DatosCuentaBancaria.kt`, `DatosIdentidad.kt`, `DatosServidorSsh.kt`, `DatosWifi.kt`, `DatosWallet.kt`, `DatosContacto.kt`.
* **Motor de Fusión y Desduplicación**:
  * [`app/src/main/java/com/jlnavas3/bovedalocal/dominio/AnalizadorDuplicadosNativos.kt`](/home/jln/BovedaLocal/app/src/main/java/com/jlnavas3/bovedalocal/dominio/AnalizadorDuplicadosNativos.kt)
  * [`app/src/main/java/com/jlnavas3/bovedalocal/dominio/AnalizadorDiscrepanciasUnificacion.kt`](/home/jln/BovedaLocal/app/src/main/java/com/jlnavas3/bovedalocal/dominio/AnalizadorDiscrepanciasUnificacion.kt)
  * [`app/src/main/java/com/jlnavas3/bovedalocal/dominio/FusionadorSubmodelosNativos.kt`](/home/jln/BovedaLocal/app/src/main/java/com/jlnavas3/bovedalocal/dominio/FusionadorSubmodelosNativos.kt)

---

## 🔒 2. Invariantes Críticos de Seguridad

1. **Cero Telemetría y Cero Red No Solicitada**: La aplicación jamás debe enviar métricas, analíticas ni conectarse a internet sin acción explícita del usuario.
2. **Compatibilidad Binaria Estricta**: Todo archivo `.bvda` generado por esta aplicación de escritorio debe poder abrirse en Android con la misma contraseña maestra, y viceversa.
3. **Seguridad en Memoria (*Zeroization*)**:
   - Toda clave derivada o contraseña maestra en Rust debe implementar el trait `Zeroize` / `ZeroizeOnDrop` del crate `zeroize`.
   - No almacenar secretos en variables globales ni strings estáticos no volátiles.

---

## 🛠️ 3. Pila Tecnológica Elegida

* **Lenguaje Principal**: **Rust** (edición 2021+).
* **Criptografía**:
  * `argon2 = "0.5"` (KDF Argon2id v0x13).
  * `aes-gcm = "0.10"` (AES-256-GCM con autenticación AAD).
  * `flate2 = "1.0"` (Compresión/descompresión Deflate para el bloque `BLZ1`).
  * `zeroize = { version = "1.8", features = ["derive"] }` (Purga de RAM).
  * `rand = "0.8"` (Generador CSPRNG).
* **Serialización**:
  * `serde = { version = "1.0", features = ["derive"] }`
  * `serde_json = "1.0"`
* **Interfaz de Usuario (GUI)**:
  * **Opción recomendada 1**: **Slint** (`slint = "1.8"`) — Estilo nativo ligero, sintaxis declarativa tipo QML, compila a binario nativo en Windows y Linux.
  * **Opción recomendada 2**: **Tauri 2.0** (`tauri`) — Rust en el backend criptográfico con interfaz moderna en HTML5/CSS/TypeScript en WebView nativo (WebView2 en Windows, WebKitGTK en Linux).
* **Sincronización P2P**:
  * `tokio` (runtime asíncrono).
  * `snow` (Noise Protocol Framework para canal E2EE local) o `rustls` (TLS 1.3 con certificados autofirmados efímeros).

---

## 📋 4. Hoja de Ruta de Implementación Sugerida

### Fase 1: Crate Core Criptográfico y Parser `.bvda`
1. Implementar la lectura y escritura de la cabecera binaria inmutable de 49 bytes.
2. Implementar la derivación Argon2id y descifrado/cifrado AES-256-GCM con AAD idéntico a la cabecera.
3. Implementar el extractor/empaquetador de compresión `BLZ1`.
4. Definir las estructuras Rust correspondientes a `ContenidoBoveda`, `Entrada`, `Categoria`, `Perfil` y los 9 tipos nativos.
5. Crear pruebas unitarias con un archivo `.bvda` real exportado desde la app de Android para validar compatibilidad bidireccional.

### Fase 2: Interfaz de Línea de Comandos (CLI) de Verificación
* Crear un binario CLI (`boveda-cli`) que permita:
  * `boveda-cli inspect archivo.bvda` (Modo Dry-Run sin desbloquear todo).
  * `boveda-cli unlock archivo.bvda` (Muestra las cuentas en terminal).
  * `boveda-cli export archivo.bvda -o salida.bvda` (Crea una copia).

### Fase 3: Interfaz Gráfica (Desktop GUI con Slint o Tauri)
* Pantalla de desbloqueo con contraseña maestra.
* Panel lateral de categorías y perfiles de identidad.
* Lista de entradas con buscador en vivo, filtros por tipo (Login, Tarjeta, SSH, etc.) y favoritos.
* Ficha de visualización de credenciales con botones para copiar usuario, contraseña, TOTP con cuenta atrás visual y visor de notas Markdown.
* Generador de contraseñas integrado (aleatorias fuertes y Diceware).

### Fase 4: Protocolo de Sincronización P2P Súper Privado
* Servidor local ligero que escuche en red Wi-Fi o mediante USB (redirección ADB).
* Emparejamiento por código QR o PIN de 6 dígitos.
* Canal E2EE cifrado con Noise Protocol o mTLS.
* Intercambio de deltas y fusión interactiva basada en el motor de discrepancias.
