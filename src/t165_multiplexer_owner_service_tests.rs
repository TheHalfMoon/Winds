use super::*;
use crate::multiplexer::domain::service::MultiplexerServiceError;
use crate::multiplexer::domain::{
    ClientSurfaceCapability, MultiplexerAuthority, MultiplexerErrorKind, MultiplexerWorkspaceId,
    PaneId, TabId,
};
use crate::persistent_runtime::domain::ClientConnectionId;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_T165_ROOT: AtomicU64 = AtomicU64::new(1);

fn test_root(label: &str) -> PathBuf {
    let id = NEXT_T165_ROOT.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("winds-t165-{label}-{}-{id}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path.canonicalize().unwrap()
}

fn runtime_root(label: &str) -> PathBuf {
    let id = NEXT_T165_ROOT.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("w165-{label}-{id}"));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path.canonicalize().unwrap()
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn start_owner(home: &Path, runtime_root: &Path, now: i64) -> PersistentOwner {
    let runtime_directory = runtime_root.join("r");
    crate::persistent_runtime::transport::unix::prepare_runtime_directory(&runtime_directory)
        .unwrap();
    PersistentOwner::start_for_test(home, &runtime_directory, now).unwrap()
}

#[cfg(windows)]
fn start_owner(home: &Path, _runtime_root: &Path, now: i64) -> PersistentOwner {
    PersistentOwner::start(home, now).unwrap()
}

fn connection(label: &str) -> ClientConnectionId {
    ClientConnectionId::new(label).unwrap()
}

fn workspace(byte: u8) -> MultiplexerWorkspaceId {
    MultiplexerWorkspaceId::from_entropy_bytes([byte; 16]).unwrap()
}

fn tab(byte: u8) -> TabId {
    TabId::from_entropy_bytes([byte; 16]).unwrap()
}

fn pane(byte: u8) -> PaneId {
    PaneId::from_entropy_bytes([byte; 16]).unwrap()
}

#[test]
fn t165_write_authority_converges_and_failed_requests_do_not_publish_state() {
    let home = test_root("authority-home");
    let runtime = runtime_root("authority-runtime");
    let mut owner = start_owner(&home, &runtime, 10);
    let first = connection("t165-first");
    let second = connection("t165-second");
    let observer = connection("t165-observer");

    let before = owner.store.load_multiplexer_topology_snapshots().unwrap();
    assert_eq!(
        owner
            .request_multiplexer_write(
                observer.clone(),
                ClientSurfaceCapability::NonInteractiveObserver,
            )
            .unwrap_err(),
        MultiplexerErrorKind::CapabilityUnavailable
    );
    assert_eq!(
        owner.multiplexer_authority(&observer),
        MultiplexerAuthority::Observer
    );
    assert_eq!(
        owner.store.load_multiplexer_topology_snapshots().unwrap(),
        before
    );

    owner
        .request_multiplexer_write(first.clone(), ClientSurfaceCapability::ControllingTerminal)
        .unwrap();
    owner
        .request_multiplexer_write(
            second.clone(),
            ClientSurfaceCapability::TrustedDesktopTerminalSurface,
        )
        .unwrap();

    let initial = owner.multiplexer_topology().generation();
    let accepted = owner
        .mutate_multiplexer_topology(&first, initial, 20, |topology, expected| {
            topology.create_workspace(
                expected,
                workspace(1),
                "accepted".to_owned(),
                tab(2),
                "main".to_owned(),
                pane(3),
            )
        })
        .unwrap();

    assert_eq!(
        owner
            .mutate_multiplexer_topology(&second, initial, 21, |topology, expected| {
                topology.rename_workspace(expected, workspace(1), "stale".to_owned())
            })
            .unwrap_err(),
        MultiplexerServiceError::Domain(MultiplexerErrorKind::StaleTopologyGeneration)
    );

    let renamed = owner
        .mutate_multiplexer_topology(&second, accepted, 30, |topology, expected| {
            topology.rename_workspace(expected, workspace(1), "durable".to_owned())
        })
        .unwrap();
    let durable = owner.store.load_multiplexer_topology_snapshots().unwrap();

    let failed = owner
        .mutate_multiplexer_topology(&second, renamed, 29, |topology, expected| {
            topology.rename_workspace(expected, workspace(1), "must-not-publish".to_owned())
        })
        .unwrap_err();
    assert!(matches!(failed, MultiplexerServiceError::Persistence(_)));
    assert_eq!(owner.multiplexer_topology().generation(), renamed);
    assert_eq!(
        owner
            .multiplexer_topology()
            .workspace(workspace(1))
            .unwrap()
            .alias,
        "durable"
    );
    assert_eq!(
        owner.store.load_multiplexer_topology_snapshots().unwrap(),
        durable
    );

    assert_eq!(
        owner
            .mutate_multiplexer_topology(&observer, renamed, 31, |topology, expected| {
                topology.rename_workspace(expected, workspace(1), "forbidden".to_owned())
            })
            .unwrap_err(),
        MultiplexerServiceError::Domain(MultiplexerErrorKind::MultiplexerWriteRequired)
    );
    assert_eq!(
        owner.release_multiplexer_write(&first),
        MultiplexerAuthority::Observer
    );
    owner.disconnect_multiplexer_client(&second);
    assert_eq!(
        owner.multiplexer_authority(&second),
        MultiplexerAuthority::Observer
    );

    drop(owner);
    let _ = fs::remove_dir_all(home);
    let _ = fs::remove_dir_all(runtime);
}

#[test]
fn t165_restart_restores_presentation_order_without_write_authority() {
    let home = test_root("restart-home");
    let runtime_one = runtime_root("restart-one");
    let client = connection("t165-restart");
    let (generation, old_owner_generation) = {
        let mut owner = start_owner(&home, &runtime_one, 40);
        let old_owner_generation = owner.generation_id();
        owner
            .request_multiplexer_write(client.clone(), ClientSurfaceCapability::ControllingTerminal)
            .unwrap();

        let mut generation = owner.multiplexer_topology().generation();
        generation = owner
            .mutate_multiplexer_topology(&client, generation, 41, |topology, expected| {
                topology.create_workspace(
                    expected,
                    workspace(11),
                    "one".to_owned(),
                    tab(12),
                    "one-tab".to_owned(),
                    pane(13),
                )
            })
            .unwrap();
        generation = owner
            .mutate_multiplexer_topology(&client, generation, 42, |topology, expected| {
                topology.create_workspace(
                    expected,
                    workspace(21),
                    "two".to_owned(),
                    tab(22),
                    "two-tab".to_owned(),
                    pane(23),
                )
            })
            .unwrap();
        generation = owner
            .mutate_multiplexer_topology(&client, generation, 43, |topology, expected| {
                topology.move_workspace(expected, workspace(21), 0)
            })
            .unwrap();
        (generation, old_owner_generation)
    };

    let runtime_two = runtime_root("restart-two");
    let owner = start_owner(&home, &runtime_two, 50);
    assert_ne!(owner.generation_id(), old_owner_generation);
    assert_eq!(owner.multiplexer_topology().generation(), generation);
    assert_eq!(
        owner
            .multiplexer_topology()
            .workspaces()
            .iter()
            .map(|item| item.id)
            .collect::<Vec<_>>(),
        vec![workspace(21), workspace(11)]
    );
    assert_eq!(
        owner.multiplexer_authority(&client),
        MultiplexerAuthority::Observer
    );

    drop(owner);
    let _ = fs::remove_dir_all(home);
    let _ = fs::remove_dir_all(runtime_one);
    let _ = fs::remove_dir_all(runtime_two);
}
