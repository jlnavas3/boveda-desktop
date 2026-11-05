/*
 * Copyright (C) 2026 Jose Luis Navas
 * Licensed under the GNU General Public License v3.0 (GPLv3).
 * See LICENSE file in the project root for full license information.
 */

use crate::sync::p2p::rfc1918::es_ip_privada_rfc1918;
use std::process::Command;

const PREFIJOS_DESCARTABLES: &[&str] = &[
    "lo", "docker", "br-", "virbr", "veth", "tun", "tap", "wg", "tailscale", "zt", "vboxnet",
    "vmnet",
];

/// Detecta las direcciones IPv4 privadas RFC 1918 candidatas de las interfaces físicas de red.
pub fn detectar_ips_lan_candidatas() -> Vec<String> {
    let mut candidatas: Vec<(u8, String)> = Vec::new();

    if let Ok(salida) = Command::new("ip")
        .args(["-o", "-4", "addr", "show"])
        .output()
    {
        if salida.status.success() {
            let texto = String::from_utf8_lossy(&salida.stdout);
            for linea in texto.lines() {
                let partes: Vec<&str> = linea.split_whitespace().collect();
                // Formato típico: "2: enp0s31f6 inet 192.168.0.210/24 ..."
                if partes.len() >= 4 && partes[2] == "inet" {
                    let iface = partes[1];
                    let ip_cidr = partes[3];
                    let ip = ip_cidr.split('/').next().unwrap_or("").trim();

                    if debe_descartar_interfaz(iface) {
                        continue;
                    }

                    if es_ip_privada_rfc1918(ip) {
                        let prioridad = prioridad_interfaz(iface);
                        candidatas.push((prioridad, ip.to_string()));
                    }
                }
            }
        }
    }

    // Ordenar por prioridad (Wi-Fi primero, luego Ethernet, etc.)
    candidatas.sort_by_key(|(p, _)| *p);

    let mut resultado = Vec::new();
    for (_, ip) in candidatas {
        if !resultado.contains(&ip) {
            resultado.push(ip);
        }
    }

    if resultado.is_empty() {
        if let Some(ip_fallback) = fallback_udp_probe() {
            resultado.push(ip_fallback);
        }
    }

    resultado
}

fn debe_descartar_interfaz(iface: &str) -> bool {
    let iface_lower = iface.to_lowercase();
    PREFIJOS_DESCARTABLES
        .iter()
        .any(|prefix| iface_lower.starts_with(prefix))
}

fn prioridad_interfaz(iface: &str) -> u8 {
    let iface_lower = iface.to_lowercase();
    // Prioridad 0: Wi-Fi física
    if iface_lower.starts_with("wlan")
        || iface_lower.starts_with("wlp")
        || iface_lower.starts_with("wls")
    {
        0
    // Prioridad 1: Ethernet física
    } else if iface_lower.starts_with("eth")
        || iface_lower.starts_with("enp")
        || iface_lower.starts_with("eno")
        || iface_lower.starts_with("ens")
    {
        1
    } else {
        2
    }
}

fn fallback_udp_probe() -> Option<String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket
        .connect("192.168.1.1:80")
        .or_else(|_| socket.connect("10.0.0.1:80"))
        .ok()?;
    let local_addr = socket.local_addr().ok()?;
    let ip = local_addr.ip().to_string();
    if es_ip_privada_rfc1918(&ip) {
        Some(ip)
    } else {
        None
    }
}
