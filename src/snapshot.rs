use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use chrono::Utc;

use crate::model::{ChangedBinding, DiffResult, PortBinding, Snapshot};

pub fn create_snapshot(bindings: Vec<PortBinding>, custom_id: Option<String>) -> Snapshot {
    let now = Utc::now();
    let id = custom_id.unwrap_or_else(|| now.format("%Y%m%d_%H%M%S").to_string());
    let hostname = std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("HOST"))
        .unwrap_or_else(|_| "localhost".to_string());

    Snapshot {
        id,
        created_at: now.to_rfc3339(),
        hostname,
        count: bindings.len(),
        bindings,
    }
}

pub fn default_snapshot_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config").join("portmapper").join("snapshots")
}

pub fn save_snapshot_file(snapshot: &Snapshot, target_path: Option<&Path>) -> Result<PathBuf, String> {
    let file_path = match target_path {
        Some(p) => p.to_path_buf(),
        None => {
            let dir = default_snapshot_dir();
            if let Err(e) = fs::create_dir_all(&dir) {
                return Err(format!("Failed to create snapshot directory {:?}: {}", dir, e));
            }
            dir.join(format!("{}.json", snapshot.id))
        }
    };

    let json_data = serde_json::to_string_pretty(snapshot)
        .map_err(|e| format!("Serialization error: {}", e))?;

    let mut file = File::create(&file_path)
        .map_err(|e| format!("Failed to create snapshot file {:?}: {}", file_path, e))?;

    file.write_all(json_data.as_bytes())
        .map_err(|e| format!("Failed to write snapshot file: {}", e))?;

    Ok(file_path)
}

pub fn load_snapshot_file(path: &Path) -> Result<Snapshot, String> {
    let mut file = File::open(path)
        .map_err(|e| format!("Failed to open snapshot {:?}: {}", path, e))?;

    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|e| format!("Failed to read snapshot file: {}", e))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse snapshot JSON: {}", e))
}

pub fn diff_snapshots(baseline: &Snapshot, current: &Snapshot) -> DiffResult {
    let mut base_map: HashMap<String, &PortBinding> = HashMap::new();
    for b in &baseline.bindings {
        base_map.insert(b.key(), b);
    }

    let mut curr_map: HashMap<String, &PortBinding> = HashMap::new();
    for b in &current.bindings {
        curr_map.insert(b.key(), b);
    }

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed = Vec::new();

    // Check additions and modifications
    for (key, curr_binding) in &curr_map {
        match base_map.get(key) {
            Some(&base_binding) => {
                if *base_binding != **curr_binding {
                    changed.push(ChangedBinding {
                        key: key.clone(),
                        port: curr_binding.port,
                        protocol: curr_binding.protocol,
                        previous: (*base_binding).clone(),
                        current: (*curr_binding).clone(),
                    });
                }
            }
            None => {
                added.push((*curr_binding).clone());
            }
        }
    }

    // Check removals
    for (key, base_binding) in &base_map {
        if !curr_map.contains_key(key) {
            removed.push((*base_binding).clone());
        }
    }

    added.sort_by_key(|b| b.port);
    removed.sort_by_key(|b| b.port);
    changed.sort_by_key(|c| c.port);

    DiffResult {
        added,
        removed,
        changed,
    }
}
