use portmapper::model::{IpVersion, PortBinding, Protocol};
use portmapper::snapshot::{create_snapshot, diff_snapshots, load_snapshot_file, save_snapshot_file};
use tempfile::tempdir;

fn mock_binding(port: u16, proto: Protocol, interface: &str, pid: Option<u32>, proc: Option<&str>) -> PortBinding {
    let is_all = interface == "0.0.0.0" || interface == "::";
    let is_local = interface == "127.0.0.1" || interface == "::1";

    PortBinding {
        port,
        protocol: proto,
        ip_version: if interface.contains(':') { IpVersion::V6 } else { IpVersion::V4 },
        interface: interface.to_string(),
        is_all_interfaces: is_all,
        is_localhost: is_local,
        pid,
        process_name: proc.map(|s| s.to_string()),
        user_id: Some(1000),
        username: Some("developer".to_string()),
    }
}

#[test]
fn test_diff_detects_newly_exposed_public_ports() {
    let base_bindings = vec![
        mock_binding(3000, Protocol::Tcp, "127.0.0.1", Some(1001), Some("node")),
    ];
    let base_snapshot = create_snapshot(base_bindings, Some("baseline".to_string()));

    let curr_bindings = vec![
        mock_binding(3000, Protocol::Tcp, "127.0.0.1", Some(1001), Some("node")),
        mock_binding(5432, Protocol::Tcp, "0.0.0.0", Some(2044), Some("postgres")), // newly exposed public port!
    ];
    let curr_snapshot = create_snapshot(curr_bindings, Some("current".to_string()));

    let diff = diff_snapshots(&base_snapshot, &curr_snapshot);

    assert!(diff.has_differences());
    assert_eq!(diff.added.len(), 1);
    assert_eq!(diff.removed.len(), 0);
    assert_eq!(diff.changed.len(), 0);

    let added = &diff.added[0];
    assert_eq!(added.port, 5432);
    assert_eq!(added.interface, "0.0.0.0");
    assert!(added.is_all_interfaces);
    assert!(diff.has_new_exposures());
}

#[test]
fn test_diff_detects_closed_ports_and_process_changes() {
    let base_bindings = vec![
        mock_binding(8080, Protocol::Tcp, "127.0.0.1", Some(1234), Some("legacy-api")),
        mock_binding(9090, Protocol::Tcp, "127.0.0.1", Some(5678), Some("prometheus")),
    ];
    let base_snapshot = create_snapshot(base_bindings, Some("baseline".to_string()));

    let curr_bindings = vec![
        // 8080 process was migrated / restarted with new PID
        mock_binding(8080, Protocol::Tcp, "127.0.0.1", Some(9999), Some("modern-api")),
        // 9090 was closed / terminated
    ];
    let curr_snapshot = create_snapshot(curr_bindings, Some("current".to_string()));

    let diff = diff_snapshots(&base_snapshot, &curr_snapshot);

    assert!(diff.has_differences());
    assert_eq!(diff.removed.len(), 1);
    assert_eq!(diff.removed[0].port, 9090);

    assert_eq!(diff.changed.len(), 1);
    assert_eq!(diff.changed[0].port, 8080);
    assert_eq!(diff.changed[0].previous.pid, Some(1234));
    assert_eq!(diff.changed[0].current.pid, Some(9999));
    assert_eq!(diff.changed[0].current.process_name.as_deref(), Some("modern-api"));
}

#[test]
fn test_snapshot_file_roundtrip() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test_snapshot.json");

    let bindings = vec![
        mock_binding(22, Protocol::Tcp, "0.0.0.0", Some(1), Some("sshd")),
        mock_binding(53, Protocol::Udp, "127.0.0.53", Some(400), Some("systemd-resolve")),
    ];
    let snapshot = create_snapshot(bindings, Some("test_run".to_string()));

    let saved_path = save_snapshot_file(&snapshot, Some(&file_path)).unwrap();
    assert_eq!(saved_path, file_path);

    let loaded = load_snapshot_file(&file_path).unwrap();
    assert_eq!(loaded.id, "test_run");
    assert_eq!(loaded.count, 2);
    assert_eq!(loaded.bindings.len(), 2);
    assert_eq!(loaded.bindings[0].port, 22);
}
