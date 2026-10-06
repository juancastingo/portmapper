# portmapper

[![CI](https://github.com/juancastingo/portmapper/actions/workflows/ci.yml/badge.svg)](https://github.com/juancastingo/portmapper/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust 1.75+](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org/)

A high-performance local network socket mapping and snapshot diffing utility written in Rust.

`portmapper` scans local TCP/UDP listening sockets, maps them to their owning PID, process name, operating system user, and interface binding scope, and provides **time-series snapshot diffing** to immediately detect when new ports are exposed or closed.

---

## Features

- 🔍 **Kernel Socket Mapping**: Reads `/proc/net/{tcp,tcp6,udp,udp6}` and correlates file descriptor socket inodes to owning processes (`/proc/[pid]/fd`) and local system usernames (`/etc/passwd`).
- 🌐 **Full Dual-Stack Support**: Identifies IPv4 and IPv6 bindings (`127.0.0.1`, `::1`, `0.0.0.0`, `::`).
- 📸 **Persistent Snapshots**: Save timestamped socket states (`portmapper save --name baseline`).
- ⚡ **Time-Series Exposure Diffing**: Compare your current environment against previous snapshots to pinpoint newly opened services or closed ports:
  - `[+] Newly Opened Ports`: Flags whether bound to localhost or public interfaces (`0.0.0.0` / `::`).
  - `[-] Closed / Terminated Ports`: Discovered shutdown services.
  - `[~] Changed Port Bindings`: Highlights process restarts or PID migrations.
- 🤖 **CI/CD Security Guard**: Exit non-zero with `--fail-on-new` to prevent development servers, databases, or test containers from inadvertently opening ports in production pipelines.
- 📦 **JSON Output**: Fully serialized machine-readable format via `--json`.

---

## Installation

### From Source (Cargo)

```bash
cargo install --git https://github.com/juancastingo/portmapper.git
```

Or build locally:

```bash
git clone https://github.com/juancastingo/portmapper.git
cd portmapper
cargo build --release
sudo cp target/release/portmapper /usr/local/bin/
```

---

## Usage

### 1. Scan Current Listening Sockets

```bash
portmapper scan
```

Output:
```text
PORT    PROTO  INTERFACE          EXPOSURE              PID      PROCESS            USER        
----------------------------------------------------------------------------------------
22      TCP    0.0.0.0            ALL (0.0.0.0)         1024     sshd               root        
80      TCP    0.0.0.0            ALL (0.0.0.0)         2450     nginx              www-data    
3000    TCP    127.0.0.1          Localhost             31294    node               developer   
5432    TCP    127.0.0.1          Localhost             1420     postgres           postgres    
```

Export structured JSON:
```bash
portmapper scan --json
```

---

### 2. Save a Baseline Snapshot

Save a named snapshot of your clean environment state:

```bash
portmapper save --name clean_baseline
```

---

### 3. Compare Current State Against Baseline

Run `diff` to instantly identify what changed:

```bash
portmapper diff
```

Output:
```text
Comparing against latest snapshot: ~/.config/portmapper/snapshots/clean_baseline.json

=== Port Exposure Snapshot Diff ===

[+] Newly Opened Ports (1) :
  + Port 8080/TCP bound to 0.0.0.0 (PUBLIC EXPOSURE (0.0.0.0 / ::)) | PID: 45192 | Proc: "python"

[-] Closed / Terminated Ports (1) :
  - Port 3000/TCP (was: 127.0.0.1, Proc: "node")
```

---

### 4. CI / Automated Audit Mode

Enforce that no unexpected public ports are introduced:

```bash
portmapper diff --baseline ./baseline.json --fail-on-new
```

---

## Commands

| Command | Description |
| :--- | :--- |
| `portmapper scan [--json]` | Inspect active listening sockets |
| `portmapper save [-n <name>] [-o <path>]` | Save a snapshot of listening ports |
| `portmapper diff [-b <baseline>] [-t <target>] [--fail-on-new]` | Diff two snapshots or live vs snapshot |
| `portmapper list` | List all saved snapshots |

---

## License

MIT License © 2026 Juan Castin
