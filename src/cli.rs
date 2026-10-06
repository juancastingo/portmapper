use clap::{Parser, Subcommand};
use colored::*;
use std::path::PathBuf;

use crate::model::{DiffResult, PortBinding};

#[derive(Parser, Debug)]
#[command(
    name = "portmapper",
    about = "Map local ports to processes, users, and interfaces with snapshot saving and time-series exposure diffing",
    version = "0.1.0"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Scan current listening ports across TCP/UDP and IPv4/IPv6
    Scan {
        /// Output results in JSON format
        #[arg(long)]
        json: bool,
    },
    /// Take a snapshot of current listening ports and save it to disk
    Save {
        /// Custom snapshot identifier or label
        #[arg(short, long)]
        name: Option<String>,
        /// Explicit file path to save snapshot to (defaults to ~/.config/portmapper/snapshots)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Compare current ports against a saved snapshot, or compare two snapshots
    Diff {
        /// Baseline snapshot file path to compare against
        #[arg(short, long)]
        baseline: Option<PathBuf>,
        /// Current snapshot file path (defaults to live socket scan if omitted)
        #[arg(short, long)]
        target: Option<PathBuf>,
        /// Output diff result in JSON format
        #[arg(long)]
        json: bool,
        /// Exit with non-zero code (1) if any newly exposed ports are discovered
        #[arg(long)]
        fail_on_new: bool,
    },
    /// List all locally saved port snapshots
    List,
}

pub fn print_bindings_table(bindings: &[PortBinding]) {
    println!(
        "\n{:<7} {:<6} {:<18} {:<14} {:<8} {:<18} {:<12}",
        "PORT", "PROTO", "INTERFACE", "EXPOSURE", "PID", "PROCESS", "USER"
    );
    println!("{}", "-".repeat(88));

    for b in bindings {
        let proto_str = format!("{}", b.protocol);
        let exposure_str = if b.is_all_interfaces {
            "ALL (0.0.0.0)".red().bold().to_string()
        } else if b.is_localhost {
            "Localhost".green().to_string()
        } else {
            "LAN / Custom".yellow().to_string()
        };

        let pid_str = b
            .pid
            .map(|p| p.to_string())
            .unwrap_or_else(|| "-".to_string());
        let proc_str = b.process_name.as_deref().unwrap_or("-");
        let user_str = b.username.as_deref().unwrap_or("-");

        println!(
            "{:<7} {:<6} {:<18} {:<23} {:<8} {:<18} {:<12}",
            b.port, proto_str, b.interface, exposure_str, pid_str, proc_str, user_str
        );
    }
    println!("\nTotal listening sockets: {}\n", bindings.len());
}

pub fn print_diff_report(diff: &DiffResult, fail_on_new: bool) {
    println!("\n=== Port Exposure Snapshot Diff ===\n");

    if !diff.has_differences() {
        println!(
            "{}",
            "✔ No changes detected. All listening sockets match baseline exactly.\n".green()
        );
        return;
    }

    if !diff.added.is_empty() {
        println!(
            "{}",
            format!("[+] Newly Opened Ports ({}) :", diff.added.len())
                .green()
                .bold()
        );
        for b in &diff.added {
            let exposure = if b.is_all_interfaces {
                "PUBLIC EXPOSURE (0.0.0.0 / ::)".red().bold().to_string()
            } else {
                "Localhost only".cyan().to_string()
            };
            println!(
                "  + Port {}/{} bound to {} ({}) | PID: {:?} | Proc: {:?}",
                b.port,
                b.protocol,
                b.interface,
                exposure,
                b.pid.unwrap_or(0),
                b.process_name.as_deref().unwrap_or("-")
            );
        }
        println!();
    }

    if !diff.removed.is_empty() {
        println!(
            "{}",
            format!("[-] Closed / Terminated Ports ({}) :", diff.removed.len())
                .red()
                .bold()
        );
        for b in &diff.removed {
            println!(
                "  - Port {}/{} (was: {}, Proc: {:?})",
                b.port,
                b.protocol,
                b.interface,
                b.process_name.as_deref().unwrap_or("-")
            );
        }
        println!();
    }

    if !diff.changed.is_empty() {
        println!(
            "{}",
            format!("[~] Changed Port Bindings ({}) :", diff.changed.len())
                .yellow()
                .bold()
        );
        for c in &diff.changed {
            println!(
                "  ~ Port {}/{}: PID {:?} -> {:?}, Proc {:?} -> {:?}",
                c.port,
                c.protocol,
                c.previous.pid,
                c.current.pid,
                c.previous.process_name.as_deref().unwrap_or("-"),
                c.current.process_name.as_deref().unwrap_or("-")
            );
        }
        println!();
    }

    if fail_on_new && diff.has_new_exposures() {
        eprintln!(
            "{}",
            "[!] CI FAILURE: Newly exposed public interfaces detected!\n"
                .red()
                .bold()
        );
    }
}
