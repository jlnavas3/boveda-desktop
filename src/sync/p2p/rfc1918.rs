/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

/// Valida de forma estricta que una dirección IP sea una IPv4 privada canónica según RFC 1918:
/// - 10.0.0.0/8 (10.0.0.0 ..= 10.255.255.255)
/// - 172.16.0.0/12 (172.16.0.0 ..= 172.31.255.255)
/// - 192.168.0.0/16 (192.168.0.0 ..= 192.168.255.255)
///
/// Prohíbe ceros a la izquierda para evitar ambigüedad octal (ej. "010.0.0.1")
/// y rechaza tajantemente loopback (127.x), link-local (169.254.x), multicast,
/// broadcast, IPs públicas y nombres de host/dominios (prevención de DNS Rebinding).
pub fn es_ip_privada_rfc1918(ip: &str) -> bool {
    let limpio = ip.trim();
    let partes: Vec<&str> = limpio.split('.').collect();
    if partes.len() != 4 {
        return false;
    }

    let mut octetos = [0u8; 4];
    for (i, seg) in partes.iter().enumerate() {
        if seg.is_empty() || seg.len() > 3 {
            return false;
        }
        // Prohibir ceros a la izquierda si tiene más de 1 dígito (evitar ambigüedad octal)
        if seg.len() > 1 && seg.starts_with('0') {
            return false;
        }
        match seg.parse::<u8>() {
            Ok(num) => octetos[i] = num,
            Err(_) => return false,
        }
    }

    let o0 = octetos[0];
    let o1 = octetos[1];

    // 10.0.0.0/8
    if o0 == 10 {
        return true;
    }

    // 172.16.0.0/12 (172.16.0.0 ..= 172.31.255.255)
    if o0 == 172 && (16..=31).contains(&o1) {
        return true;
    }

    // 192.168.0.0/16
    if o0 == 192 && o1 == 168 {
        return true;
    }

    false
}
