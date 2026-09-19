use super::{
    WindowsNamedPipeClient, WindowsNamedPipeServer, WindowsTransportError, entropy_128_with,
    generate_owner_generation_id, generate_runtime_namespace_id, pipe_name_for_generation,
};
use crate::persistent_runtime::domain::OwnerGenerationId;
use crate::persistent_runtime::peer::current_process_user_sid;
use std::thread;
use windows_sys::Win32::Foundation::{ERROR_ACCESS_DENIED, GetLastError};
use windows_sys::Win32::Security::{ImpersonateAnonymousToken, RevertToSelf};
use windows_sys::Win32::System::Threading::GetCurrentThread;

fn generation(byte: u8) -> OwnerGenerationId {
    OwnerGenerationId::from_entropy_bytes([byte; 16]).expect("valid generation")
}

struct AnonymousImpersonation;

impl AnonymousImpersonation {
    fn begin() -> Self {
        // SAFETY: GetCurrentThread returns the pseudo-handle for the calling thread.
        let result = unsafe { ImpersonateAnonymousToken(GetCurrentThread()) };
        assert_ne!(result, 0, "anonymous impersonation failed: {}", unsafe {
            GetLastError()
        });
        Self
    }
}

impl Drop for AnonymousImpersonation {
    fn drop(&mut self) {
        // SAFETY: this guard exists only after successful anonymous impersonation.
        assert_ne!(unsafe { RevertToSelf() }, 0, "RevertToSelf failed");
    }
}

#[test]
fn t150_pipe_name_is_current_user_and_generation_bound_without_network_surface() {
    let sid = current_process_user_sid().unwrap();
    let generation = generation(2);
    let name = pipe_name_for_generation(&sid, generation).unwrap();
    assert!(name.starts_with(r"\\.\pipe\winds-runtime-v1-u"));
    assert!(name.contains(&sid.canonical_hex()));
    assert!(name.ends_with(&generation.as_hex()));
    assert!(!name.contains("tcp"));
    assert!(!name.contains("http"));
}

#[test]
fn t150_effective_dacl_same_user_round_trip_and_peer_sid_proof_succeed() {
    let generation = generation(3);
    let mut server = WindowsNamedPipeServer::bind(generation).unwrap();
    server.validate_effective_security().unwrap();
    let expected_name = server.pipe_name().to_owned();

    let client = thread::spawn(move || {
        let client = WindowsNamedPipeClient::connect(generation).unwrap();
        assert_eq!(client.pipe_name(), expected_name);
        client.write_all(b"ping").unwrap();
        let mut response = [0_u8; 4];
        client.read_exact(&mut response).unwrap();
        assert_eq!(&response, b"pong");
    });

    server.accept_same_user().unwrap();
    let mut request = [0_u8; 4];
    server.read_exact(&mut request).unwrap();
    assert_eq!(&request, b"ping");
    server.write_all(b"pong").unwrap();
    client.join().unwrap();
}

#[test]
fn t150_anonymous_principal_is_denied_by_the_actual_pipe_dacl() {
    let generation = generation(4);
    let _server = WindowsNamedPipeServer::bind(generation).unwrap();
    let _anonymous = AnonymousImpersonation::begin();
    assert_eq!(
        WindowsNamedPipeClient::connect(generation).err().unwrap(),
        WindowsTransportError::PrincipalDenied
    );
    assert_eq!(unsafe { GetLastError() }, ERROR_ACCESS_DENIED);
}

#[test]
fn t150_same_generation_collision_and_wrong_generation_fail_closed() {
    let active_generation = generation(5);
    let _server = WindowsNamedPipeServer::bind(active_generation).unwrap();
    assert_eq!(
        WindowsNamedPipeServer::bind(active_generation)
            .err()
            .unwrap(),
        WindowsTransportError::PipeNameCollision
    );
    assert_eq!(
        WindowsNamedPipeClient::connect(generation(6))
            .err()
            .unwrap(),
        WindowsTransportError::ExpectedGenerationUnavailable
    );
}

#[test]
fn t150_bcrypt_entropy_contract_rejects_short_failure_and_zero_results() {
    assert_eq!(
        entropy_128_with(|bytes| {
            bytes.fill(7);
            Ok(bytes.len() - 1)
        })
        .unwrap_err(),
        WindowsTransportError::EntropyShortRead
    );
    assert_eq!(
        entropy_128_with(|_| Err(WindowsTransportError::EntropyUnavailable)).unwrap_err(),
        WindowsTransportError::EntropyUnavailable
    );
    assert_eq!(
        entropy_128_with(|bytes| Ok(bytes.len())).unwrap_err(),
        WindowsTransportError::EntropyInvalid
    );

    assert_ne!(
        generate_runtime_namespace_id().unwrap().as_hex(),
        "00000000000000000000000000000000"
    );
    assert_ne!(
        generate_owner_generation_id().unwrap().as_hex(),
        "00000000000000000000000000000000"
    );
}

#[test]
fn t150_surface_contains_no_loopback_wsl_owner_pty_or_service_installation() {
    let source = include_str!("persistent_runtime/transport/windows.rs");
    for prohibited in [
        "TcpListener",
        "TcpStream",
        "UdpSocket",
        "WebSocket",
        "127.0.0.1",
        "portable_pty",
        "CreateProcess",
        "ServiceControlManager",
        "wsl.exe",
    ] {
        assert!(!source.contains(prohibited), "{prohibited}");
    }
}
