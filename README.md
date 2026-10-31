# Bóveda Local Desktop

[![Language](https://img.shields.io/badge/Language-Rust-orange.svg?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![GUI](https://img.shields.io/badge/GUI-Slint%20%2F%20Tauri-blue.svg)](https://slint.dev/)
[![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20Windows-brightgreen.svg)]()
[![Security](https://img.shields.io/badge/Security-Zero--Knowledge%20%7C%20Zero--Network-blueviolet.svg)]()
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

**Bóveda Local Desktop** es el cliente de escritorio oficial y soberano (Linux y Windows) diseñado para coexistir, interoperar y sincronizarse de forma privada con la aplicación móvil **Bóveda Local** para Android.

---

## 🎯 Visión y Filosofía

1. **Soberanía y Confianza Cero en Nube (Zero-Knowledge)**: Tus contraseñas jamás pasan por servidores externos, nubes de terceros ni telemetría comercial.
2. **Compatibilidad Nativa con Archivos `.bvda`**: Capacidad total para abrir, inspeccionar, editar y empaquetar bóvedas cifradas interoperables con la app de Android.
3. **Seguridad en Memoria con Rust**: Eliminación matemática de desbordamientos de búfer (*buffer overflow*), punteros colgantes y sobreescritura estricta de memoria (*zeroization*) para proteger las claves maestras.
4. **Sincronización P2P Súper Privada**: Conexión directa por red Wi-Fi local o cable USB mediante canales cifrados de extremo a extremo sin depender de internet.

---

## 📂 Guías de Arquitectura e Implementación

Para desarrolladores y agentes de inteligencia artificial:

* 📖 **[INSTRUCCIONES_AGENTE.md](INSTRUCCIONES_AGENTE.md)**: Manual de referencia con el roadmap, directivas de desarrollo y enlaces al código fuente del proyecto hermano Android en `/home/jln/BovedaLocal`.
* 🔐 **[ESPECIFICACION_FORMATO_BVDA.md](ESPECIFICACION_FORMATO_BVDA.md)**: Especificación detallada a nivel de byte de la cabecera de 49 bytes, derivación Argon2id, AES-256-GCM, compresión BLZ1 (Deflate) y modelos JSON.

---

## 🛠️ Pila Tecnológica

* **Lenguaje**: Rust (Edición 2021)
* **Criptografía**: `argon2`, `aes-gcm`, `zeroize`
* **Compresión**: `flate2` (Deflate)
* **Serialización**: `serde`, `serde_json`
* **GUI Desktop**: Slint / Tauri
* **Red Local P2P**: `tokio`, `snow` (Noise Protocol) / `rustls`

---

## 📄 Licencia

Este proyecto se distribuye bajo la licencia **GNU General Public License v3.0 (GPLv3)** en armonía con el ecosistema de Bóveda Local.
