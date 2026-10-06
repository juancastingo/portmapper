use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::net::{Ipv4Addr, Ipv6Addr};

use crate::model::{IpVersion, PortBinding, Protocol};

pub fn scan_listening_ports() -> Vec<PortBinding> {
    let mut bindings = Vec::new();

    // Map inode -> (pid, process_name)
    let inode_map = build_inode_process_map();
    // Map uid -> username
    let user_map = build_user_map();

    // TCP IPv4
    parse_proc_net(
        "/proc/net/tcp",
        Protocol::Tcp,
        IpVersion::V4,
        &inode_map,
        &user_map,
        &mut bindings,
    );
    // TCP IPv6
    parse_proc_net(
        "/proc/net/tcp6",
        Protocol::Tcp,
        IpVersion::V6,
        &inode_map,
        &user_map,
        &mut bindings,
    );
    // UDP IPv4
    parse_proc_net(
        "/proc/net/udp",
        Protocol::Udp,
        IpVersion::V4,
        &inode_map,
        &user_map,
        &mut bindings,
    );
    // UDP IPv6
    parse_proc_net(
        "/proc/net/udp6",
        Protocol::Udp,
        IpVersion::V6,
        &inode_map,
        &user_map,
        &mut bindings,
    );

    // Sort by port, then protocol
    bindings.sort_by(|a, b| a.port.cmp(&b.port).then_with(|| a.protocol.to_string().cmp(&b.protocol.to_string())));
    bindings
}

fn parse_proc_net(
    path: &str,
    protocol: Protocol,
    ip_version: IpVersion,
    inode_map: &HashMap<u64, (u32, String)>,
    user_map: &HashMap<u32, String>,
    results: &mut Vec<PortBinding>,
) {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return,
    };

    let reader = BufReader::new(file);

    for (idx, line_res) in reader.lines().enumerate() {
        if idx == 0 {
            continue; // Header line
        }
        let line = match line_res {
            Ok(l) => l,
            Err(_) => continue,
        };

        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 10 {
            continue;
        }

        // Field 1: local_address (hex_ip:hex_port)
        // Field 3: st (state) - For TCP, 0A is LISTEN. For UDP, 07 is UDP state.
        let state = fields[3];
        if protocol == Protocol::Tcp && state != "0A" {
            continue;
        }

        let local_addr = fields[1];
        let addr_parts: Vec<&str> = local_addr.split(':').collect();
        if addr_parts.len() != 2 {
            continue;
        }

        let hex_ip = addr_parts[0];
        let hex_port = addr_parts[1];

        let port = match u16::from_str_radix(hex_port, 16) {
            Ok(p) => p,
            Err(_) => continue,
        };

        let ip_str = match ip_version {
            IpVersion::V4 => parse_hex_ipv4(hex_ip),
            IpVersion::V6 => parse_hex_ipv6(hex_ip),
        };

        let uid: Option<u32> = fields[7].parse().ok();
        let inode: u64 = fields[9].parse().unwrap_or(0);

        let (pid, process_name) = if inode > 0 {
            match inode_map.get(&inode) {
                Some((p, name)) => (Some(*p), Some(name.clone())),
                None => (None, None),
            }
        } else {
            (None, None)
        };

        let username = uid.and_then(|u| user_map.get(&u).cloned());

        let is_all = ip_str == "0.0.0.0" || ip_str == "::";
        let is_local = ip_str == "127.0.0.1" || ip_str == "::1";

        results.push(PortBinding {
            port,
            protocol,
            ip_version,
            interface: ip_str,
            is_all_interfaces: is_all,
            is_localhost: is_local,
            pid,
            process_name,
            user_id: uid,
            username,
        });
    }
}

fn parse_hex_ipv4(hex: &str) -> String {
    if hex.len() != 8 {
        return hex.to_string();
    }
    if let Ok(num) = u32::from_str_radix(hex, 16) {
        // Little endian byte order
        let ip = Ipv4Addr::from(num.to_ne_bytes());
        return ip.to_string();
    }
    hex.to_string()
}

fn parse_hex_ipv6(hex: &str) -> String {
    if hex.len() != 32 {
        return hex.to_string();
    }
    let mut bytes = [0u8; 16];
    for i in 0..4 {
        let chunk = &hex[i * 8..(i + 1) * 8];
        if let Ok(word) = u32::from_str_radix(chunk, 16) {
            let chunk_bytes = word.to_ne_bytes();
            bytes[i * 4..(i + 1) * 4].copy_from_slice(&chunk_bytes);
        }
    }
    let ip = Ipv6Addr::from(bytes);
    ip.to_string()
}

fn build_inode_process_map() -> HashMap<u64, (u32, String)> {
    let mut map = HashMap::new();
    let proc_dir = match fs::read_dir("/proc") {
        Ok(d) => d,
        Err(_) => return map,
    };

    for entry in proc_dir.flatten() {
        let file_name = entry.file_name();
        let pid_str = file_name.to_string_lossy();
        let pid: u32 = match pid_str.parse() {
            Ok(p) => p,
            Err(_) => continue,
        };

        let process_name = fs::read_to_string(format!("/proc/{}/comm", pid))
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        let fd_dir = format!("/proc/{}/fd", pid);
        if let Ok(fds) = fs::read_dir(fd_dir) {
            for fd_entry in fds.flatten() {
                if let Ok(target) = fs::read_link(fd_entry.path()) {
                    let target_str = target.to_string_lossy();
                    if target_str.starts_with("socket:[") && target_str.ends_with(']') {
                        let inode_str = &target_str[8..target_str.len() - 1];
                        if let Ok(inode) = inode_str.parse::<u64>() {
                            map.insert(inode, (pid, process_name.clone()));
                        }
                    }
                }
            }
        }
    }

    map
}

fn build_user_map() -> HashMap<u32, String> {
    let mut map = HashMap::new();
    if let Ok(file) = File::open("/etc/passwd") {
        let reader = BufReader::new(file);
        for line in reader.lines().map_while(Result::ok) {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 3 {
                let username = parts[0];
                if let Ok(uid) = parts[2].parse::<u32>() {
                    map.insert(uid, username.to_string());
                }
            }
        }
    }
    map
}
