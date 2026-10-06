use std::fs;
use std::process;
use clap::Parser;

use portmapper::cli::{print_bindings_table, print_diff_report, Cli, Commands};
use portmapper::scanner::scan_listening_ports;
use portmapper::snapshot::{
    create_snapshot, default_snapshot_dir, diff_snapshots, load_snapshot_file, save_snapshot_file,
};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan { json } => {
            let bindings = scan_listening_ports();
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&bindings).unwrap_or_else(|_| "[]".to_string())
                );
            } else {
                print_bindings_table(&bindings);
            }
        }
        Commands::Save { name, output } => {
            let bindings = scan_listening_ports();
            let snapshot = create_snapshot(bindings, name);
            match save_snapshot_file(&snapshot, output.as_deref()) {
                Ok(path) => {
                    println!("✔ Snapshot successfully saved to: {}", path.display());
                    println!("  Total sockets mapped: {}", snapshot.count);
                }
                Err(err) => {
                    eprintln!("Error saving snapshot: {}", err);
                    process::exit(1);
                }
            }
        }
        Commands::Diff {
            baseline,
            target,
            json,
            fail_on_new,
        } => {
            let base_snapshot = match baseline {
                Some(p) => match load_snapshot_file(&p) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Failed to load baseline snapshot {:?}: {}", p, e);
                        process::exit(1);
                    }
                },
                None => {
                    // Try to find the latest snapshot in default dir
                    let dir = default_snapshot_dir();
                    let latest = match find_latest_snapshot(&dir) {
                        Some(p) => p,
                        None => {
                            eprintln!("No baseline specified and no previous snapshots found in {}", dir.display());
                            eprintln!("Run `portmapper save` first to create a baseline snapshot.");
                            process::exit(1);
                        }
                    };
                    println!("Comparing against latest snapshot: {}", latest.display());
                    match load_snapshot_file(&latest) {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("Failed to load latest snapshot: {}", e);
                            process::exit(1);
                        }
                    }
                }
            };

            let curr_snapshot = match target {
                Some(p) => match load_snapshot_file(&p) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Failed to load target snapshot {:?}: {}", p, e);
                        process::exit(1);
                    }
                },
                None => {
                    let bindings = scan_listening_ports();
                    create_snapshot(bindings, Some("live_scan".to_string()))
                }
            };

            let diff = diff_snapshots(&base_snapshot, &curr_snapshot);

            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&diff).unwrap_or_else(|_| "{}".to_string())
                );
            } else {
                print_diff_report(&diff, fail_on_new);
            }

            if fail_on_new && diff.has_new_exposures() {
                process::exit(2);
            }
        }
        Commands::List => {
            let dir = default_snapshot_dir();
            println!("\nSnapshots in {}:", dir.display());
            if let Ok(entries) = fs::read_dir(&dir) {
                let mut files = Vec::new();
                for entry in entries.flatten() {
                    if entry.path().extension().and_then(|e| e.to_str()) == Some("json") {
                        files.push(entry.path());
                    }
                }
                files.sort();
                if files.is_empty() {
                    println!("  (No snapshots saved yet. Run `portmapper save` to create one)");
                } else {
                    for f in files {
                        println!("  • {}", f.file_name().unwrap_or_default().to_string_lossy());
                    }
                }
            } else {
                println!("  (Snapshot directory does not exist yet)");
            }
            println!();
        }
    }
}

fn find_latest_snapshot(dir: &std::path::Path) -> Option<std::path::PathBuf> {
    let entries = fs::read_dir(dir).ok()?;
    let mut files: Vec<std::path::PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
        .collect();
    files.sort();
    files.pop()
}
