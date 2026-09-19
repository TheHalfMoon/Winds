use crate::persistent_runtime::domain::{OwnerGenerationId, RuntimeNamespaceId};
use crate::persistent_runtime::peer::{
    WindowsUserSid, current_process_user_sid, require_same_user_named_pipe_client,
    validate_pipe_owner_and_dacl,
};
use std::ffi::c_void;
use std::mem::size_of;
use std::ptr::{null, null_mut};
use windows_sys::Win32::Foundation::{
    CloseHandle, ERROR_ACCESS_DENIED, ERROR_FILE_NOT_FOUND, ERROR_PIPE_BUSY, ERROR_PIPE_CONNECTED,
    GENERIC_READ, GENERIC_WRITE, GetLastError, HANDLE, INVALID_HANDLE_VALUE, LocalFree,
};
use windows_sys::Win32::Security::Authorization::{
    ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
};
use windows_sys::Win32::Security::Cryptography::{
    BCRYPT_USE_SYSTEM_PREFERRED_RNG, BCryptGenRandom,
};
use windows_sys::Win32::Security::{PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES};
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, FILE_FLAG_FIRST_PIPE_INSTANCE, OPEN_EXISTING, PIPE_ACCESS_DUPLEX, READ_CONTROL,
    ReadFile, WriteFile,
};
use windows_sys::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe, PIPE_READMODE_BYTE,
    PIPE_REJECT_REMOTE_CLIENTS, PIPE_TYPE_BYTE, PIPE_WAIT,
};

const PIPE_PREFIX: &str = r"\\.\pipe\winds-runtime-v1";
const MAX_PIPE_NAME_UTF16_UNITS: usize = 240;
const MAX_SECURITY_DESCRIPTOR_UTF16_UNITS: usize = 1024;
const PIPE_BUFFER_BYTES: u32 = 64 * 1024;
const IDENTITY_ENTROPY_BYTES: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WindowsTransportError {
    PipeNameTooLong,
    PipeNameCollision,
    ExpectedGenerationUnavailable,
    PipeNotConnected,
    PipeAlreadyConnected,
    PipeClosed,
    PrincipalDenied,
    PrincipalProofUnavailable,
    EffectiveSecurityMismatch,
    SecurityDescriptorUnavailable,
    ImpersonationRevertFailed,
    EntropyUnavailable,
    EntropyShortRead,
    EntropyInvalid,
    Win32(u32),
}

fn last_error_code() -> u32 {
    // SAFETY: GetLastError has no preconditions and reads thread-local state.
    unsafe { GetLastError() }
}

struct OwnedPipeHandle(HANDLE);

impl OwnedPipeHandle {
    fn new(handle: HANDLE) -> Result<Self, WindowsTransportError> {
        if handle.is_null() || handle == INVALID_HANDLE_VALUE {
            Err(WindowsTransportError::Win32(last_error_code()))
        } else {
            Ok(Self(handle))
        }
    }

    fn raw(&self) -> HANDLE {
        self.0
    }
}

impl Drop for OwnedPipeHandle {
    fn drop(&mut self) {
        if !self.0.is_null() && self.0 != INVALID_HANDLE_VALUE {
            // SAFETY: this wrapper owns the handle and closes it exactly once.
            let _ = unsafe { CloseHandle(self.0) };
        }
    }
}

struct LocalSecurityDescriptor(PSECURITY_DESCRIPTOR);

impl LocalSecurityDescriptor {
    fn for_current_user(user_sid: &WindowsUserSid) -> Result<Self, WindowsTransportError> {
        let sid = user_sid.to_sddl_string()?;
        let sddl = format!("O:{sid}D:P(A;;GA;;;{sid})");
        let wide = security_descriptor_wide_nul(&sddl)?;
        let mut descriptor: PSECURITY_DESCRIPTOR = null_mut();
        // SAFETY: wide is NUL-terminated UTF-16 and descriptor is a valid writable output pointer.
        let converted = unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                wide.as_ptr(),
                SDDL_REVISION_1,
                &mut descriptor,
                null_mut(),
            )
        };
        if converted == 0 || descriptor.is_null() {
            return Err(WindowsTransportError::SecurityDescriptorUnavailable);
        }
        Ok(Self(descriptor))
    }

    fn as_mut_ptr(&self) -> *mut c_void {
        self.0
    }
}

impl Drop for LocalSecurityDescriptor {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: ConvertStringSecurityDescriptorToSecurityDescriptorW allocates with LocalAlloc.
            let _ = unsafe { LocalFree(self.0) };
        }
    }
}

fn pipe_name_wide_nul(value: &str) -> Result<Vec<u16>, WindowsTransportError> {
    if value.contains('\0') {
        return Err(WindowsTransportError::PipeNameTooLong);
    }
    let mut wide: Vec<u16> = value.encode_utf16().collect();
    if wide.len() + 1 > MAX_PIPE_NAME_UTF16_UNITS {
        return Err(WindowsTransportError::PipeNameTooLong);
    }
    wide.push(0);
    Ok(wide)
}

fn security_descriptor_wide_nul(value: &str) -> Result<Vec<u16>, WindowsTransportError> {
    if value.contains('\0') {
        return Err(WindowsTransportError::SecurityDescriptorUnavailable);
    }
    let mut wide: Vec<u16> = value.encode_utf16().collect();
    if wide.len() + 1 > MAX_SECURITY_DESCRIPTOR_UTF16_UNITS {
        return Err(WindowsTransportError::SecurityDescriptorUnavailable);
    }
    wide.push(0);
    Ok(wide)
}

pub(crate) fn pipe_name_for_generation(
    user_sid: &WindowsUserSid,
    owner_generation_id: OwnerGenerationId,
) -> Result<String, WindowsTransportError> {
    let sid_hex = user_sid.canonical_hex();
    let name = format!("{PIPE_PREFIX}-u{sid_hex}-g{}", owner_generation_id.as_hex());
    let _ = pipe_name_wide_nul(&name)?;
    Ok(name)
}

fn map_server_create_error(code: u32) -> WindowsTransportError {
    if code == ERROR_ACCESS_DENIED || code == ERROR_PIPE_BUSY {
        WindowsTransportError::PipeNameCollision
    } else {
        WindowsTransportError::Win32(code)
    }
}

fn map_client_connect_error(code: u32) -> WindowsTransportError {
    match code {
        ERROR_ACCESS_DENIED => WindowsTransportError::PrincipalDenied,
        ERROR_FILE_NOT_FOUND => WindowsTransportError::ExpectedGenerationUnavailable,
        ERROR_PIPE_BUSY => WindowsTransportError::PipeNameCollision,
        _ => WindowsTransportError::Win32(code),
    }
}

pub(crate) struct WindowsNamedPipeServer {
    handle: OwnedPipeHandle,
    pipe_name: String,
    user_sid: WindowsUserSid,
    connected: bool,
}

impl WindowsNamedPipeServer {
    pub(crate) fn bind(
        owner_generation_id: OwnerGenerationId,
    ) -> Result<Self, WindowsTransportError> {
        let user_sid = current_process_user_sid()?;
        let pipe_name = pipe_name_for_generation(&user_sid, owner_generation_id)?;
        let pipe_name_wide = pipe_name_wide_nul(&pipe_name)?;
        let descriptor = LocalSecurityDescriptor::for_current_user(&user_sid)?;
        let security_attributes = SECURITY_ATTRIBUTES {
            nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: descriptor.as_mut_ptr(),
            bInheritHandle: 0,
        };

        // SAFETY: pipe_name_wide is NUL-terminated and security_attributes points to a live
        // descriptor for the duration of CreateNamedPipeW. All flags are bounded constants.
        let handle = unsafe {
            CreateNamedPipeW(
                pipe_name_wide.as_ptr(),
                PIPE_ACCESS_DUPLEX | FILE_FLAG_FIRST_PIPE_INSTANCE,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
                1,
                PIPE_BUFFER_BYTES,
                PIPE_BUFFER_BYTES,
                0,
                &security_attributes,
            )
        };
        if handle == INVALID_HANDLE_VALUE || handle.is_null() {
            return Err(map_server_create_error(last_error_code()));
        }
        let handle = OwnedPipeHandle::new(handle)?;
        validate_pipe_owner_and_dacl(handle.raw(), &user_sid)?;

        Ok(Self {
            handle,
            pipe_name,
            user_sid,
            connected: false,
        })
    }

    pub(crate) fn pipe_name(&self) -> &str {
        &self.pipe_name
    }

    pub(crate) fn validate_effective_security(&self) -> Result<(), WindowsTransportError> {
        validate_pipe_owner_and_dacl(self.handle.raw(), &self.user_sid)
    }

    pub(crate) fn accept_same_user(&mut self) -> Result<(), WindowsTransportError> {
        if self.connected {
            return Err(WindowsTransportError::PipeAlreadyConnected);
        }
        // SAFETY: handle is a valid named-pipe server handle and synchronous connect passes null OVERLAPPED.
        let connected = unsafe { ConnectNamedPipe(self.handle.raw(), null_mut()) };
        if connected == 0 {
            let error = last_error_code();
            if error != ERROR_PIPE_CONNECTED {
                return Err(WindowsTransportError::Win32(error));
            }
        }
        if let Err(error) = require_same_user_named_pipe_client(self.handle.raw(), &self.user_sid) {
            // SAFETY: a client is connected at this point; disconnect is best-effort on denial.
            let _ = unsafe { DisconnectNamedPipe(self.handle.raw()) };
            return Err(error);
        }
        self.connected = true;
        Ok(())
    }

    pub(crate) fn read_exact(&self, buffer: &mut [u8]) -> Result<(), WindowsTransportError> {
        if !self.connected {
            return Err(WindowsTransportError::PipeNotConnected);
        }
        read_exact_handle(self.handle.raw(), buffer)
    }

    pub(crate) fn write_all(&self, buffer: &[u8]) -> Result<(), WindowsTransportError> {
        if !self.connected {
            return Err(WindowsTransportError::PipeNotConnected);
        }
        write_all_handle(self.handle.raw(), buffer)
    }
}

impl Drop for WindowsNamedPipeServer {
    fn drop(&mut self) {
        if self.connected {
            // SAFETY: handle remains valid until the owned handle field is dropped after this method.
            let _ = unsafe { DisconnectNamedPipe(self.handle.raw()) };
        }
    }
}

pub(crate) struct WindowsNamedPipeClient {
    handle: OwnedPipeHandle,
    pipe_name: String,
}

impl WindowsNamedPipeClient {
    pub(crate) fn connect(
        expected_owner_generation_id: OwnerGenerationId,
    ) -> Result<Self, WindowsTransportError> {
        let user_sid = current_process_user_sid()?;
        let pipe_name = pipe_name_for_generation(&user_sid, expected_owner_generation_id)?;
        let pipe_name_wide = pipe_name_wide_nul(&pipe_name)?;
        // SAFETY: name is NUL-terminated; null security/template pointers are permitted for CreateFileW.
        let handle = unsafe {
            CreateFileW(
                pipe_name_wide.as_ptr(),
                GENERIC_READ | GENERIC_WRITE | READ_CONTROL,
                0,
                null(),
                OPEN_EXISTING,
                0,
                null_mut(),
            )
        };
        if handle == INVALID_HANDLE_VALUE || handle.is_null() {
            return Err(map_client_connect_error(last_error_code()));
        }
        let handle = OwnedPipeHandle::new(handle)?;
        validate_pipe_owner_and_dacl(handle.raw(), &user_sid)?;
        Ok(Self { handle, pipe_name })
    }

    pub(crate) fn pipe_name(&self) -> &str {
        &self.pipe_name
    }

    pub(crate) fn read_exact(&self, buffer: &mut [u8]) -> Result<(), WindowsTransportError> {
        read_exact_handle(self.handle.raw(), buffer)
    }

    pub(crate) fn write_all(&self, buffer: &[u8]) -> Result<(), WindowsTransportError> {
        write_all_handle(self.handle.raw(), buffer)
    }
}

fn read_exact_handle(handle: HANDLE, buffer: &mut [u8]) -> Result<(), WindowsTransportError> {
    let mut offset = 0_usize;
    while offset < buffer.len() {
        let chunk_len = (buffer.len() - offset).min(u32::MAX as usize) as u32;
        let mut read = 0_u32;
        // SAFETY: buffer slice is writable for chunk_len bytes; synchronous ReadFile passes null OVERLAPPED.
        if unsafe {
            ReadFile(
                handle,
                buffer[offset..].as_mut_ptr(),
                chunk_len,
                &mut read,
                null_mut(),
            )
        } == 0
        {
            return Err(WindowsTransportError::Win32(last_error_code()));
        }
        if read == 0 {
            return Err(WindowsTransportError::PipeClosed);
        }
        offset += read as usize;
    }
    Ok(())
}

fn write_all_handle(handle: HANDLE, buffer: &[u8]) -> Result<(), WindowsTransportError> {
    let mut offset = 0_usize;
    while offset < buffer.len() {
        let chunk_len = (buffer.len() - offset).min(u32::MAX as usize) as u32;
        let mut written = 0_u32;
        // SAFETY: buffer slice is readable for chunk_len bytes; synchronous WriteFile passes null OVERLAPPED.
        if unsafe {
            WriteFile(
                handle,
                buffer[offset..].as_ptr(),
                chunk_len,
                &mut written,
                null_mut(),
            )
        } == 0
        {
            return Err(WindowsTransportError::Win32(last_error_code()));
        }
        if written == 0 {
            return Err(WindowsTransportError::PipeClosed);
        }
        offset += written as usize;
    }
    Ok(())
}

fn entropy_128_with<F>(mut fill: F) -> Result<[u8; IDENTITY_ENTROPY_BYTES], WindowsTransportError>
where
    F: FnMut(&mut [u8]) -> Result<usize, WindowsTransportError>,
{
    let mut bytes = [0_u8; IDENTITY_ENTROPY_BYTES];
    let written = fill(&mut bytes)?;
    if written != IDENTITY_ENTROPY_BYTES {
        return Err(WindowsTransportError::EntropyShortRead);
    }
    if bytes.iter().all(|byte| *byte == 0) {
        return Err(WindowsTransportError::EntropyInvalid);
    }
    Ok(bytes)
}

fn os_entropy_128() -> Result<[u8; IDENTITY_ENTROPY_BYTES], WindowsTransportError> {
    entropy_128_with(|bytes| {
        // SAFETY: bytes is writable for exactly bytes.len() bytes; system-preferred RNG uses no algorithm handle.
        let status = unsafe {
            BCryptGenRandom(
                null_mut(),
                bytes.as_mut_ptr(),
                bytes.len() as u32,
                BCRYPT_USE_SYSTEM_PREFERRED_RNG,
            )
        };
        if status != 0 {
            Err(WindowsTransportError::EntropyUnavailable)
        } else {
            Ok(bytes.len())
        }
    })
}

pub(crate) fn generate_runtime_namespace_id() -> Result<RuntimeNamespaceId, WindowsTransportError> {
    RuntimeNamespaceId::from_entropy_bytes(os_entropy_128()?)
        .map_err(|_| WindowsTransportError::EntropyInvalid)
}

pub(crate) fn generate_owner_generation_id() -> Result<OwnerGenerationId, WindowsTransportError> {
    OwnerGenerationId::from_entropy_bytes(os_entropy_128()?)
        .map_err(|_| WindowsTransportError::EntropyInvalid)
}

#[cfg(test)]
#[path = "../../t150_windows_private_transport_tests.rs"]
mod t150_windows_private_transport_tests;
