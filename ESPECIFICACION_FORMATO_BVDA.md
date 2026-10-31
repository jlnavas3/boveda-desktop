# Especificación Técnica del Formato de Archivo `.bvda`

El archivo `.bvda` (*Bóveda Local Data Archive*) es el formato canónico de almacenamiento cifrado y respaldo utilizado por Bóveda Local. Su diseño combina cifrado autenticado de grado militar con compresión transparente y resistencia demostrada contra ataques de hardware masivo (GPU/ASIC).

---

## 1. Disposición Binaria General

```text
+-------------------------------------------------------------------------+
| Cabecera Inmutable Autenticada (AAD)                                    |
| Exactamente 49 Bytes (Big-Endian)                                       |
+-------------------------------------------------------------------------+
| Cuerpo Cifrado (Ciphertext)                                             |
| Longitud variable: AES-256-GCM sobre el bloque comprimido (BLZ1)        |
+-------------------------------------------------------------------------+
| Etiqueta de Autenticación GCM (Tag)                                     |
| Exactamente 16 Bytes (128 bits)                                         |
+-------------------------------------------------------------------------+
```

---

## 2. Desglose Detallado de la Cabecera (49 Bytes)

La cabecera precede inmediatamente al cuerpo cifrado y se procesa en orden de red (**Big-Endian**):

| Offset (Bytes) | Tamaño | Nombre de Campo | Tipo / Formato | Valor / Descripción |
| :---: | :---: | :--- | :--- | :--- |
| `00..03` | 4 B | `MAGIC` | Bytes ASCII | Constante `0x42 0x56 0x44 0x41` (Texto: `"BVDA"`). Identifica el archivo. |
| `04` | 1 B | `VERSION` | `uint8` | Versión del formato de contenedor binario. Actualmente `0x01`. |
| `05..20` | 16 B | `SALT` | Bytes binarios | Sal criptográfica aleatoria de 128 bits generada por CSPRNG (`SecureRandom`). |
| `21..24` | 4 B | `MEMORY_KIB` | `uint32_be` | Memoria RAM requerida por Argon2id expresada en KiB (Ej. `65536` = 64 MiB, `131072` = 128 MiB, `262144` = 256 MiB). |
| `25..28` | 4 B | `ITERATIONS` | `uint32_be` | Número de pases o iteraciones temporales de Argon2id (Ej. `3`, `4`, `6`). |
| `29..32` | 4 B | `PARALLELISM` | `uint32_be` | Número de hilos o lanes de paralelismo de Argon2id (Por defecto `4`). |
| `33..36` | 4 B | `HASH_LENGTH` | `uint32_be` | Longitud en bytes de la clave simétrica generada. Siempre `32` (256 bits). |
| `37..48` | 12 B | `NONCE` | Bytes binarios | Vector de Inicialización (IV) de 96 bits generado por CSPRNG para AES-256-GCM. |

> **IMPORTANTE (Protección de Integridad AAD):**
> Los 49 bytes íntegros de esta cabecera se pasan a la primitiva AES-GCM como **Additional Authenticated Data (AAD)** mediante `cipher.updateAAD(cabecera)`.
> Si un atacante modifica un solo bit de la cabecera (por ejemplo, alterando `MEMORY_KIB` para acelerar un ataque de fuerza bruta), la verificación del Tag GCM fallará irremisiblemente y el archivo no será descifrado.

---

## 3. Derivación de Clave (KDF): Argon2id

* **Variante:** `Argon2id` (versión `0x13`), combinando la resistencia de Argon2d contra ataques GPU/side-channel y de Argon2i contra timing-attacks.
* **Entradas:**
  * Password: `password.as_bytes()`
  * Salt: 16 bytes extraídos de `SALT`
  * Memory: Extraído de `MEMORY_KIB` (en KiB)
  * Iterations: Extraído de `ITERATIONS`
  * Parallelism: Extraído de `PARALLELISM`
  * Output Key Length: 32 bytes (256 bits)

En Rust se implementa directamente con el crate oficial `argon2`:
```rust
use argon2::{Argon2, Algorithm, Version, Params};

let params = Params::new(memory_kib, iterations, parallelism, Some(32))
    .map_err(|_| "Parámetros Argon2 inválidos")?;
let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

let mut key = [0u8; 32];
argon2.hash_password_into(password, salt, &mut key)
    .map_err(|_| "Fallo en KDF Argon2id")?;
```

---

## 4. Cifrado y Autenticación: AES-256-GCM

* **Algoritmo:** `AES-256-GCM` (`AES/GCM/NoPadding`).
* **Clave:** Clave de 32 bytes derivada en el paso anterior.
* **Nonce (IV):** 12 bytes extraídos de la cabecera.
* **AAD:** Los primeros 49 bytes exactos del archivo `.bvda`.
* **Tag:** Los últimos 16 bytes del archivo.

En Rust se implementa con el crate `aes-gcm`:
```rust
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::{Aead, Payload};

let cipher = Aes256Gcm::new_from_slice(&key)?;
let nonce = Nonce::from_slice(&cabecera.nonce);

let payload = Payload {
    msg: &cuerpo_cifrado, // Bytes desde el offset 49 hasta el final (incluye tag al final en aes-gcm crate)
    aad: &cabecera_bytes, // Los 49 bytes de la cabecera
};

let decrypted_bytes = cipher.decrypt(nonce, payload)?;
```

---

## 5. Compresión Transparente (Bloque BLZ1)

Los bytes descifrados `decrypted_bytes` corresponden al payload comprimido:
* Si los primeros 4 bytes son `0x42 0x4C 0x5A 0x31` (ASCII: `"BLZ1"`):
  * El bloque a partir del byte 4 en adelante está comprimido con **Deflate (zlib standard)**.
  * Debe descomprimirse con `flate2::read::DeflateDecoder` o `flate2::read::ZlibDecoder`.
* Si los primeros 4 bytes NO son `"BLZ1"`:
  * El bloque ya es texto plano JSON (retrocompatibilidad con copias históricas).

En Rust con `flate2`:
```rust
use flate2::read::DeflateDecoder;
use std::io::Read;

const MAGIC_BLZ1: &[u8; 4] = b"BLZ1";

let json_bytes = if decrypted_bytes.starts_with(MAGIC_BLZ1) {
    let mut decoder = DeflateDecoder::new(&decrypted_bytes[4..]);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    decompressed
} else {
    decrypted_bytes
};

let json_str = String::from_utf8(json_bytes)?;
```

---

## 6. Esquema JSON de Datos (`ContenidoBoveda`)

El JSON resultante decodificado es un objeto UTF-8 con la siguiente estructura:

```json
{
  "version": 1,
  "entradas": [
    {
      "id": "uuid-v4-string",
      "titulo": "Google",
      "usuario": "usuario@gmail.com",
      "contrasena": "ClaveSecreta123!",
      "urls": ["https://accounts.google.com"],
      "notas": "Notas en formato Markdown con **negrita** o tablas.",
      "categoriaId": "id-categoria-o-null",
      "favorito": true,
      "anclado": false,
      "secretoTotp": "JBSWY3DPEHPK3PXP",
      "algoritmoTotp": "SHA1",
      "digitosTotp": 6,
      "periodoTotp": 30,
      "totpsSecundarios": [],
      "tipo": "LOGIN",
      "camposPersonalizados": [],
      "etiquetas": ["trabajo", "personal"],
      "adjuntos": [],
      "passkey": null,
      "perfilId": null,
      "creadaEn": 1700000000000,
      "modificadaEn": 1700000000000,
      "datosTarjeta": null,
      "datosCuentaBancaria": null,
      "datosIdentidad": null,
      "datosServidor": null,
      "datosWifi": null,
      "datosWallet": null,
      "datosContacto": null
    }
  ],
  "papelera": [],
  "colecciones": [
    {
      "id": "cat-uuid",
      "nombre": "Servicios Web",
      "color": -16744448,
      "icono": "Globe",
      "creadaEn": 1700000000000,
      "modificadaEn": 1700000000000
    }
  ],
  "identidades": [
    {
      "id": "perfil-uuid",
      "nombre": "Personal",
      "color": -123456,
      "orden": 0
    }
  ],
  "configuracion": {
    "prefijosSubdominios": ["api", "auth", "login"],
    "tldsDescartables": ["local", "internal", "lan"],
    "marcasPersonalizadas": {},
    "puertosServiciosLocales": {"8006": "Proxmox", "8123": "Home Assistant"},
    "rutasServiciosLocales": {"/admin": "Panel Admin", "/pihole": "Pi-hole"},
    "octetosRouter": [1, 254],
    "plantillasCamposPersonalizadas": [],
    "autoBloqueoSegundos": 600,
    "frenoIntentosGratis": 3,
    "frenoSegundosMax": 300,
    "autodestruccionIntentosFallidosMax": 10,
    "autofillSugerenciasTeclado": true,
    "correoBaseAlias": "",
    "delimitadorAlias": "+",
    "longitudSufijoAlias": 6
  }
}
```

---

## 7. Mapeo de Submodelos Nativos (`tipo`)

El campo `tipo` de cada entrada contiene uno de los siguientes valores enum:

1. `"LOGIN"`: Solo utiliza campos base.
2. `"NOTA"`: Utiliza `titulo` y `notas` (Markdown con tablas).
3. `"TARJETA"`: Utiliza `datosTarjeta`:
   ```json
   {
     "numeroTarjeta": "4532123456789012",
     "titular": "Juan Pérez",
     "fechaCaducidad": "12/28",
     "cvv": "123",
     "tipoTarjeta": "CREDITO"
   }
   ```
4. `"CUENTA_BANCARIA"`: Utiliza `datosCuentaBancaria`:
   ```json
   {
     "banco": "Banco Santander",
     "iban": "ES9121000418450200051332",
     "swiftBic": "BSCHESMMXXX",
     "titular": "Juan Pérez"
   }
   ```
5. `"IDENTIDAD"`: Utiliza `datosIdentidad`:
   ```json
   {
     "tipoDocumento": "DNI",
     "numero": "12345678Z",
     "pais": "España",
     "fechaEmision": "2020-01-15",
     "fechaExpiracion": "2030-01-15"
   }
   ```
6. `"SERVIDOR"`: Utiliza `datosServidor`:
   ```json
   {
     "host": "192.168.1.100",
     "puerto": "22",
     "usuario": "root",
     "clavePrivada": "-----BEGIN OPENSSH PRIVATE KEY-----\n...",
     "clavePublica": "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5..."
   }
   ```
7. `"WIFI"`: Utiliza `datosWifi`:
   ```json
   {
     "ssid": "MiFibra_5G",
     "seguridad": "WPA3",
     "oculta": false
   }
   ```
8. `"WALLET"`: Utiliza `datosWallet`:
   ```json
   {
     "fraseSemilla": "word1 word2 ... word12",
     "clavePublica": "0x71C...456",
     "red": "Ethereum"
   }
   ```
9. `"CONTACTO"`: Utiliza `datosContacto`:
   ```json
   {
     "nombre": "María López",
     "telefono": "+34612345678",
     "correo": "maria@ejemplo.com",
     "empresa": "Acme Corp"
   }
   ```
