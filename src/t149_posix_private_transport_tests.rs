use super::{
    BoundUnixListener, ENDPOINT_MODE, MAX_PORTABLE_UNIX_SOCKET_PATH_BYTES, RUNTIME_DIRECTORY_MODE,
    connect_same_user, endpoint_path, entropy_128_with, generate_owner_generation_id,
    generate_runtime_namespace_id, prepare_runtime_directory, resolve_runtime_directory_from,
    validate_directory_facts, validate_endpoint_facts,
};
use crate::persistent_runtime::peer::{
    current_effective_uid, peer_effective_uid, require_same_effective_user,
};
use crate::persistent_runtime::transport::PosixTransportError;
use std::fs::{self, File, Permissions};
use std::io::{Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{FileTypeExt, PermissionsExt, symlink};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

struct TestRoot(PathBuf);

impl TestRoot {
    fn new(label: &str) -> Self {
        let serial = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "winds-t149-{label}-{}-{serial}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir(&path).unwrap();
        fs::set_permissions(&path, Permissions::from_mode(0o700)).unwrap();
        Self(path.canonicalize().unwrap())
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn runtime_dir(&self) -> PathBuf {
        self.0.join("runtime")
    }
}

impl Drop for TestRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn chmod(path: &Path, mode: u32) {
    fs::set_permissions(path, Permissions::from_mode(mode)).unwrap();
}

#[test]
fn t149_resolver_is_absolute_deterministic_and_has_bounded_fallback() {
    let preferred = Path::new("/tmp");
    let fallback = Path::new("/tmp");

    let first = resolve_runtime_directory_from(preferred, fallback, 4242).unwrap();
    let second = resolve_runtime_directory_from(preferred, fallback, 4242).unwrap();
    assert_eq!(first, second);
    assert!(first.is_absolute());
    assert_eq!(
        first,
        preferred.canonicalize().unwrap().join("winds-runtime-v1")
    );

    let long = PathBuf::from(format!(
        "/tmp/{}",
        "x".repeat(MAX_PORTABLE_UNIX_SOCKET_PATH_BYTES)
    ));
    let selected = resolve_runtime_directory_from(&long, fallback, 4242).unwrap();
    assert_eq!(
        selected,
        fallback.canonicalize().unwrap().join("winds-runtime-4242")
    );
    assert!(
        endpoint_path(&selected)
            .unwrap()
            .as_os_str()
            .as_bytes()
            .len()
            <= 100
    );
}

#[test]
fn t149_runtime_directory_is_user_owned_0700_and_rejects_symlink_mode_and_type_confusion() {
    let root = TestRoot::new("directory");
    let runtime = root.runtime_dir();
    prepare_runtime_directory(&runtime).unwrap();
    let metadata = fs::symlink_metadata(&runtime).unwrap();
    assert_eq!(
        metadata.permissions().mode() & 0o777,
        RUNTIME_DIRECTORY_MODE
    );

    chmod(&runtime, 0o770);
    assert_eq!(
        prepare_runtime_directory(&runtime).unwrap_err(),
        PosixTransportError::RuntimeDirectoryModeMismatch
    );
    chmod(&runtime, 0o700);

    assert_eq!(
        validate_directory_facts(
            current_effective_uid().wrapping_add(1),
            0o700,
            current_effective_uid()
        )
        .unwrap_err(),
        PosixTransportError::RuntimeDirectoryOwnershipMismatch
    );

    fs::remove_dir(&runtime).unwrap();
    let target = root.path().join("target");
    fs::create_dir(&target).unwrap();
    chmod(&target, 0o700);
    symlink(&target, &runtime).unwrap();
    assert_eq!(
        prepare_runtime_directory(&runtime).unwrap_err(),
        PosixTransportError::RuntimeDirectorySymlink
    );
}

#[test]
fn t149_runtime_directory_rejects_ancestor_symlink_aliases() {
    let root = TestRoot::new("alias");
    let real_parent = root.path().join("real-parent");
    fs::create_dir(&real_parent).unwrap();
    chmod(&real_parent, 0o700);
    let alias_parent = root.path().join("alias-parent");
    symlink(&real_parent, &alias_parent).unwrap();
    let aliased_runtime = alias_parent.join("runtime");

    assert_eq!(
        prepare_runtime_directory(&aliased_runtime).unwrap_err(),
        PosixTransportError::RuntimeDirectoryPathAlias
    );
    assert!(!real_parent.join("runtime").exists());
}

#[test]
fn t149_same_user_kernel_peer_proof_succeeds_and_wrong_uid_fails_closed() {
    let (left, right) = UnixStream::pair().unwrap();
    let expected = current_effective_uid();
    assert_eq!(peer_effective_uid(&left).unwrap(), expected);
    assert_eq!(peer_effective_uid(&right).unwrap(), expected);
    assert!(require_same_effective_user(expected, expected).is_ok());
    assert_eq!(
        require_same_effective_user(expected, expected.wrapping_add(1)).unwrap_err(),
        PosixTransportError::PrincipalDenied
    );
}

#[test]
fn t149_secure_listener_and_client_prove_same_user_and_exchange_bytes() {
    let root = TestRoot::new("happy");
    let runtime = root.runtime_dir();
    let listener = BoundUnixListener::bind(&runtime).unwrap();
    let endpoint = listener.endpoint_path().to_path_buf();
    let metadata = fs::symlink_metadata(&endpoint).unwrap();
    assert_eq!(metadata.permissions().mode() & 0o777, ENDPOINT_MODE);

    let join = thread::spawn(move || {
        let mut stream = listener.accept_same_user().unwrap();
        let mut request = [0_u8; 4];
        stream.read_exact(&mut request).unwrap();
        assert_eq!(&request, b"ping");
        stream.write_all(b"pong").unwrap();
    });

    let mut client = connect_same_user(&runtime).unwrap();
    client.write_all(b"ping").unwrap();
    let mut response = [0_u8; 4];
    client.read_exact(&mut response).unwrap();
    assert_eq!(&response, b"pong");
    join.join().unwrap();
}

#[test]
fn t149_live_socket_collision_is_preserved_and_never_replaced() {
    let root = TestRoot::new("live");
    let runtime = root.runtime_dir();
    prepare_runtime_directory(&runtime).unwrap();
    let endpoint = endpoint_path(&runtime).unwrap();
    let live = UnixListener::bind(&endpoint).unwrap();
    chmod(&endpoint, 0o600);

    assert_eq!(
        BoundUnixListener::bind(&runtime).unwrap_err(),
        PosixTransportError::LiveEndpointCollision
    );
    assert!(endpoint.exists());
    drop(live);
}

#[test]
fn t149_stale_socket_is_removed_only_after_refused_connect_and_rebound() {
    let root = TestRoot::new("stale");
    let runtime = root.runtime_dir();
    prepare_runtime_directory(&runtime).unwrap();
    let endpoint = endpoint_path(&runtime).unwrap();
    let stale = UnixListener::bind(&endpoint).unwrap();
    chmod(&endpoint, 0o600);
    drop(stale);
    assert!(endpoint.exists());

    let rebound = BoundUnixListener::bind(&runtime).unwrap();
    assert_eq!(rebound.endpoint_path(), endpoint);
}

#[test]
fn t149_listener_drop_does_not_delete_a_replacement_socket() {
    let root = TestRoot::new("drop");
    let runtime = root.runtime_dir();
    let listener = BoundUnixListener::bind(&runtime).unwrap();
    let endpoint = listener.endpoint_path().to_path_buf();
    let moved = runtime.join("original.sock");
    fs::rename(&endpoint, &moved).unwrap();

    let replacement = UnixListener::bind(&endpoint).unwrap();
    chmod(&endpoint, 0o600);
    drop(listener);

    assert!(
        fs::symlink_metadata(&endpoint)
            .unwrap()
            .file_type()
            .is_socket()
    );
    let connected = UnixStream::connect(&endpoint).unwrap();
    drop(connected);
    drop(replacement);
}

#[test]
fn t149_regular_file_and_symlink_endpoint_collisions_are_preserved() {
    let root = TestRoot::new("col");
    let runtime = root.runtime_dir();
    prepare_runtime_directory(&runtime).unwrap();
    let endpoint = endpoint_path(&runtime).unwrap();

    File::create(&endpoint).unwrap();
    assert_eq!(
        BoundUnixListener::bind(&runtime).unwrap_err(),
        PosixTransportError::EndpointCollision
    );
    assert!(endpoint.is_file());
    fs::remove_file(&endpoint).unwrap();

    let target = runtime.join("target");
    File::create(&target).unwrap();
    symlink(&target, &endpoint).unwrap();
    assert_eq!(
        BoundUnixListener::bind(&runtime).unwrap_err(),
        PosixTransportError::EndpointSymlink
    );
    assert!(
        fs::symlink_metadata(&endpoint)
            .unwrap()
            .file_type()
            .is_symlink()
    );
}

#[test]
fn t149_endpoint_mode_and_owner_facts_fail_closed() {
    let uid = current_effective_uid();
    assert_eq!(
        validate_endpoint_facts(uid, 0o660, uid).unwrap_err(),
        PosixTransportError::EndpointModeMismatch
    );
    assert_eq!(
        validate_endpoint_facts(uid.wrapping_add(1), 0o600, uid).unwrap_err(),
        PosixTransportError::EndpointOwnershipMismatch
    );
}

#[test]
fn t149_entropy_contract_rejects_short_failed_and_zero_results() {
    assert_eq!(
        entropy_128_with(|bytes| {
            bytes.fill(7);
            Ok(bytes.len() - 1)
        })
        .unwrap_err(),
        PosixTransportError::EntropyShortRead
    );
    assert_eq!(
        entropy_128_with(|_| Err(PosixTransportError::EntropyUnavailable)).unwrap_err(),
        PosixTransportError::EntropyUnavailable
    );
    assert_eq!(
        entropy_128_with(|bytes| Ok(bytes.len())).unwrap_err(),
        PosixTransportError::EntropyInvalid
    );

    let runtime_id = generate_runtime_namespace_id().unwrap();
    let generation_id = generate_owner_generation_id().unwrap();
    assert_ne!(runtime_id.to_string(), "00000000000000000000000000000000");
    assert_ne!(
        generation_id.to_string(),
        "00000000000000000000000000000000"
    );
}

#[test]
fn t149_transport_surface_has_no_tcp_http_websocket_windows_pipe_or_process_owner() {
    let unix_source = include_str!("persistent_runtime/transport/unix.rs");
    let peer_source = include_str!("persistent_runtime/peer.rs");
    for prohibited in [
        "TcpListener",
        "TcpStream",
        "WebSocket",
        "http::",
        "NamedPipe",
        "portable_pty",
        "std::process::Command",
    ] {
        assert!(!unix_source.contains(prohibited), "{prohibited}");
        assert!(!peer_source.contains(prohibited), "{prohibited}");
    }
}
