/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

//! Generador criptográfico de contraseñas de alta entropía y frases de paso (Diceware).
//! Utiliza el generador CSPRNG del kernel del sistema operativo (`rand::rngs::OsRng`).

use rand::rngs::OsRng;
use rand::seq::SliceRandom;
use rand::Rng;
use zeroize::Zeroizing;

const LETRAS_MINUSCULAS: &str = "abcdefghijklmnopqrstuvwxyz";
const LETRAS_MAYUSCULAS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const NUMEROS: &str = "0123456789";
const SIMBOLOS: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?";

const CARACTERES_AMBIGUOS: &[char] = &['l', '1', 'I', 'o', '0', 'O'];

/// Configuración personalizable para la generación de contraseñas o frases de paso.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasswordGeneratorConfig {
    pub longitud: usize,
    pub mayusculas: bool,
    pub minusculas: bool,
    pub numeros: bool,
    pub simbolos: bool,
    pub evitar_ambiguos: bool,
    pub modo_frase: bool,
    pub numero_palabras: usize,
    pub separador: String,
    pub capitalizar_palabras: bool,
}

impl Default for PasswordGeneratorConfig {
    fn default() -> Self {
        Self {
            longitud: 20,
            mayusculas: true,
            minusculas: true,
            numeros: true,
            simbolos: true,
            evitar_ambiguos: true,
            modo_frase: false,
            numero_palabras: 4,
            separador: "-".to_string(),
            capitalizar_palabras: true,
        }
    }
}

/// Genera una contraseña o frase de paso segura con protección física de memoria RAM.
pub fn generar_contrasena(config: &PasswordGeneratorConfig) -> Zeroizing<String> {
    let mut rng = OsRng;

    if config.modo_frase {
        generar_frase_diceware(config, &mut rng)
    } else {
        generar_contrasena_caracteres(config, &mut rng)
    }
}

fn generar_contrasena_caracteres(config: &PasswordGeneratorConfig, rng: &mut OsRng) -> Zeroizing<String> {
    let filtrar_ambiguos = |chars: &str| -> Vec<char> {
        chars
            .chars()
            .filter(|c| !config.evitar_ambiguos || !CARACTERES_AMBIGUOS.contains(c))
            .collect()
    };

    let mut grupos: Vec<Vec<char>> = Vec::new();
    if config.minusculas {
        let chars = filtrar_ambiguos(LETRAS_MINUSCULAS);
        if !chars.is_empty() {
            grupos.push(chars);
        }
    }
    if config.mayusculas {
        let chars = filtrar_ambiguos(LETRAS_MAYUSCULAS);
        if !chars.is_empty() {
            grupos.push(chars);
        }
    }
    if config.numeros {
        let chars = filtrar_ambiguos(NUMEROS);
        if !chars.is_empty() {
            grupos.push(chars);
        }
    }
    if config.simbolos {
        let chars = filtrar_ambiguos(SIMBOLOS);
        if !chars.is_empty() {
            grupos.push(chars);
        }
    }

    // Si no se seleccionó ninguna opción válida, fallback seguro a alfanumérico
    if grupos.is_empty() {
        grupos.push(filtrar_ambiguos(LETRAS_MINUSCULAS));
        grupos.push(filtrar_ambiguos(NUMEROS));
    }

    let longitud = config.longitud.clamp(8, 128);
    let mut resultado: Vec<char> = Vec::with_capacity(longitud);

    // Garantizar al menos un carácter de cada grupo seleccionado
    for grupo in &grupos {
        if resultado.len() < longitud {
            if let Some(&c) = grupo.choose(rng) {
                resultado.push(c);
            }
        }
    }

    // Pool global unificado para rellenar la longitud restante
    let mut pool_completo: Vec<char> = Vec::new();
    for g in &grupos {
        pool_completo.extend_from_slice(g);
    }

    while resultado.len() < longitud {
        if let Some(&c) = pool_completo.choose(rng) {
            resultado.push(c);
        }
    }

    // Barajar aleatoriamente para no dejar fijos los primeros caracteres
    resultado.shuffle(rng);

    Zeroizing::new(resultado.into_iter().collect())
}

fn generar_frase_diceware(config: &PasswordGeneratorConfig, rng: &mut OsRng) -> Zeroizing<String> {
    let num_palabras = config.numero_palabras.clamp(3, 8);
    let mut palabras_elegidas = Vec::with_capacity(num_palabras);

    for _ in 0..num_palabras {
        let idx = rng.gen_range(0..LISTA_DICEWARE_SOBERANA.len());
        let mut palabra = LISTA_DICEWARE_SOBERANA[idx].to_string();

        if config.capitalizar_palabras {
            if let Some(primer) = palabra.chars().next() {
                let resto: String = palabra.chars().skip(1).collect();
                palabra = format!("{}{}", primer.to_uppercase(), resto);
            }
        }

        palabras_elegidas.push(palabra);
    }

    let frase = palabras_elegidas.join(&config.separador);
    Zeroizing::new(frase)
}

/// Evalúa la fuerza y entropía aproximada de una contraseña, retornando (puntuación 0..100, etiqueta).
pub fn calcular_fuerza_contrasena(pass: &str) -> (i32, &'static str) {
    if pass.is_empty() {
        return (0, "Vacía");
    }

    let mut puntuacion = 0;
    let len = pass.len();

    // Longitud
    if len >= 8 { puntuacion += 20; }
    if len >= 10 { puntuacion += 15; }
    if len >= 12 { puntuacion += 15; }
    if len >= 16 { puntuacion += 20; }
    if len >= 20 { puntuacion += 15; }

    // Variedad de caracteres
    let tiene_minus = pass.chars().any(|c| c.is_ascii_lowercase());
    let tiene_mayus = pass.chars().any(|c| c.is_ascii_uppercase());
    let tiene_num = pass.chars().any(|c| c.is_ascii_digit());
    let tiene_simbolo = pass.chars().any(|c| !c.is_alphanumeric());

    let mut tipos = 0;
    if tiene_minus { tipos += 1; }
    if tiene_mayus { tipos += 1; }
    if tiene_num { tipos += 1; }
    if tiene_simbolo { tipos += 1; }

    match tipos {
        4 => puntuacion += 25,
        3 => puntuacion += 15,
        2 => puntuacion += 10,
        _ => {}
    }

    let puntuacion = puntuacion.min(100);

    let etiqueta = match puntuacion {
        0..=39 => "Débil",
        40..=69 => "Media",
        70..=89 => "Fuerte",
        _ => "Ultra-Segura",
    };

    (puntuacion, etiqueta)
}

/// Lista Diceware soberana de palabras limpias y memorables en español (sin tildes para interoperabilidad universal).
const LISTA_DICEWARE_SOBERANA: &[&str] = &[
    "aguila", "alarma", "aldea", "alfil", "ancla", "arbol", "arena", "astro",
    "atomo", "aurora", "avellana", "bahia", "baliza", "bambu", "barco", "bosque",
    "boveda", "brujula", "brisa", "cabina", "cactus", "caoba", "capitel", "cascada",
    "cedro", "cenit", "cielo", "cipres", "clima", "colina", "cometa", "cristal",
    "cuarzo", "cumbre", "delfin", "delta", "diamante", "duna", "eco", "eclipse",
    "edificio", "energia", "esfera", "esmeralda", "espejo", "espiga", "espuela", "estela",
    "faro", "fiordo", "flecha", "flauta", "fortaleza", "fresno", "fuente", "galaxia",
    "gaviota", "glaciar", "golondrina", "granito", "guitarra", "halcon", "helice", "horizonte",
    "huerto", "huracan", "iceberg", "iman", "impulso", "isla", "jardin", "jaspe",
    "jungla", "ladera", "laguna", "lampara", "laurel", "lienzo", "limon", "lince",
    "linterna", "llama", "lluvia", "lucero", "luna", "marea", "marfil", "melodia",
    "meteorito", "mirador", "molino", "montana", "mosaico", "motor", "muralla", "neblina",
    "nieve", "nimbus", "navio", "nube", "oasis", "oceano", "olivo", "oraculo",
    "orbita", "oriente", "palacio", "palmera", "pampa", "panal", "parque", "pastizal",
    "pennon", "petalo", "piedra", "pinar", "planeta", "platino", "polar", "puente",
    "puerto", "quijote", "radar", "rafaga", "raiz", "rampa", "rayo", "refugio",
    "reloj", "resina", "rio", "roca", "rocio", "roble", "rueda", "sabana",
    "safiro", "sauce", "selva", "semilla", "sendero", "sideral", "silueta", "solsticio",
    "sombra", "sonido", "talud", "tempestad", "timon", "torre", "trebol", "trueno",
    "tulipan", "tundra", "turbina", "universo", "valle", "vapor", "veleta", "ventisca",
    "vibora", "viento", "viaducto", "volcan", "vortice", "zafiro", "zenit", "zocalo",
    "abrazo", "acorde", "albor", "alcazar", "alce", "alfiler", "algarrobo", "alhaja",
    "alisio", "almendra", "alondra", "alpino", "altar", "ambar", "anhelo", "antorcha",
    "arcilla", "arrecife", "artico", "asombro", "atalaya", "aura", "azahar", "azimut",
    "bailarin", "baluarte", "bandada", "barril", "bastion", "batuta", "beduino", "bisonte",
    "bitacora", "bonanza", "bordado", "borrasca", "bravura", "brezo", "brote", "brujula",
    "caballero", "cadencia", "caliza", "camino", "campana", "cantera", "cantaro", "caverna",
    "cebada", "centella", "ceruleo", "chopo", "ciclon", "cima", "circulo", "clarin"
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generar_contrasena_longitud_y_caracteres() {
        let config = PasswordGeneratorConfig {
            longitud: 24,
            mayusculas: true,
            minusculas: true,
            numeros: true,
            simbolos: true,
            evitar_ambiguos: false,
            modo_frase: false,
            ..Default::default()
        };

        let pass = generar_contrasena(&config);
        assert_eq!(pass.len(), 24);
        assert!(pass.chars().any(|c| c.is_ascii_lowercase()));
        assert!(pass.chars().any(|c| c.is_ascii_uppercase()));
        assert!(pass.chars().any(|c| c.is_ascii_digit()));
        assert!(pass.chars().any(|c| !c.is_alphanumeric()));
    }

    #[test]
    fn test_evitar_ambiguos() {
        let config = PasswordGeneratorConfig {
            longitud: 64,
            mayusculas: true,
            minusculas: true,
            numeros: true,
            simbolos: false,
            evitar_ambiguos: true,
            modo_frase: false,
            ..Default::default()
        };

        for _ in 0..10 {
            let pass = generar_contrasena(&config);
            for c in CARACTERES_AMBIGUOS {
                assert!(!pass.contains(*c), "No debe contener caracter ambiguo '{}'", c);
            }
        }
    }

    #[test]
    fn test_generar_passphrase_diceware() {
        let config = PasswordGeneratorConfig {
            modo_frase: true,
            numero_palabras: 5,
            separador: ".".to_string(),
            capitalizar_palabras: true,
            ..Default::default()
        };

        let pass = generar_contrasena(&config);
        let partes: Vec<&str> = pass.split('.').collect();
        assert_eq!(partes.len(), 5);
        for p in partes {
            assert!(!p.is_empty());
            assert!(p.chars().next().unwrap().is_ascii_uppercase());
        }
    }

    #[test]
    fn test_calculo_fuerza() {
        let (p1, e1) = calcular_fuerza_contrasena("123");
        assert!(p1 < 40);
        assert_eq!(e1, "Débil");

        let (p2, e2) = calcular_fuerza_contrasena("Passw0rdSegura2026!#");
        assert!(p2 >= 70);
        assert!(matches!(e2, "Fuerte" | "Ultra-Segura"));
    }
}
