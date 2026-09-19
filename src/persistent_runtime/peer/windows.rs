use crate::persistent_runtime::transport::windows::WindowsTransportError;
use std::ffi::c_void;
use std::mem::size_of;
use std::ptr::null_mut;
use windows_sys::Win32::Foundation::{
    CloseHandle, ERROR_INSUFFICIENT_BUFFER, ERROR_SUCCESS, GENERIC_ALL, GetLastError, HANDLE,
    LocalFree,
};
use windows_sys::Win32::Security::Authorization::{
    ConvertSidToStringSidW, GetSecurityInfo, SE_KERNEL_OBJECT,
};
use windows_sys::Win32::Security::{
    ACCESS_ALLOWED_ACE, ACE_HEADER, CopySid, DACL_SECURITY_INFORMATION, EqualSid, GetAce,
    GetLengthSid, GetSecurityDescriptorControl, GetTokenInformation, IsValidSid,
    OWNER_SECURITY_INFORMATION, PSID, RevertToSelf, SE_DACL_PROTECTED, SECURITY_MAX_SID_SIZE,
    TOKEN_QUERY, TOKEN_USER, TokenUser,
};
use windows_sys::Win32::System::Pipes::ImpersonateNamedPipeClient;
use windows_sys::Win32::System::Threading::{
    GetCurrentProcess, GetCurrentThread, OpenProcessToken, OpenThreadToken,
};
use windows_sys::core::PWSTR;

const MAX_TOKEN_USER_BYTES: u32 = 1024;
const MAX_SID_STRING_UNITS: usize = 256;
const ACCESS_ALLOWED_ACE_TYPE_VALUE: u8 = 0;

struct OwnedHandle(HANDLE);

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: this wrapper owns the token handle and closes it exactly once.
            let _ = unsafe { CloseHandle(self.0) };
        }
    }
}

struct LocalAllocation(*mut c_void);

impl Drop for LocalAllocation {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: the pointer came from a Win32 API documented to require LocalFree.
            let _ = unsafe { LocalFree(self.0) };
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct WindowsUserSid {
    storage: Vec<usize>,
    byte_len: u32,
}

impl WindowsUserSid {
    fn copy_from(source: PSID) -> Result<Self, WindowsTransportError> {
        if source.is_null() {
            return Err(WindowsTransportError::PrincipalProofUnavailable);
        }
        // SAFETY: source is supplied by a successful Windows token/security API call.
        if unsafe { IsValidSid(source) } == 0 {
            return Err(WindowsTransportError::PrincipalProofUnavailable);
        }
        // SAFETY: source is a validated SID.
        let byte_len = unsafe { GetLengthSid(source) };
        if byte_len == 0 || byte_len > SECURITY_MAX_SID_SIZE {
            return Err(WindowsTransportError::PrincipalProofUnavailable);
        }
        let words = (byte_len as usize).div_ceil(size_of::<usize>());
        let mut storage = vec![0_usize; words];
        let destination = storage.as_mut_ptr().cast::<c_void>();
        // SAFETY: destination is aligned, writable, and at least byte_len bytes; source is valid.
        if unsafe { CopySid(byte_len, destination, source) } == 0 {
            return Err(WindowsTransportError::Win32(unsafe { GetLastError() }));
        }
        Ok(Self { storage, byte_len })
    }

    pub(crate) fn as_psid(&self) -> PSID {
        self.storage.as_ptr().cast_mut().cast::<c_void>()
    }

    pub(crate) fn matches(&self, other: PSID) -> bool {
        if other.is_null() {
            return false;
        }
        // SAFETY: self owns a valid SID; other is validated before comparison by EqualSid.
        unsafe { IsValidSid(other) != 0 && EqualSid(self.as_psid(), other) != 0 }
    }

    pub(crate) fn canonical_hex(&self) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        // SAFETY: storage owns a validated SID and byte_len is its exact GetLengthSid size.
        let bytes = unsafe {
            std::slice::from_raw_parts(self.as_psid().cast::<u8>(), self.byte_len as usize)
        };
        let mut output = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            output.push(HEX[(byte >> 4) as usize] as char);
            output.push(HEX[(byte & 0x0f) as usize] as char);
        }
        output
    }

    pub(crate) fn to_sddl_string(&self) -> Result<String, WindowsTransportError> {
        let mut raw: PWSTR = null_mut();
        // SAFETY: self contains a valid copied SID and raw is a writable output pointer.
        if unsafe { ConvertSidToStringSidW(self.as_psid(), &mut raw) } == 0 {
            return Err(last_error());
        }
        if raw.is_null() {
            return Err(WindowsTransportError::PrincipalProofUnavailable);
        }
        let allocation = LocalAllocation(raw.cast());
        let mut units = Vec::new();
        for index in 0..MAX_SID_STRING_UNITS {
            // SAFETY: ConvertSidToStringSidW returns a NUL-terminated string; reads are bounded.
            let unit = unsafe { *raw.add(index) };
            if unit == 0 {
                return String::from_utf16(&units)
                    .map_err(|_| WindowsTransportError::PrincipalProofUnavailable);
            }
            units.push(unit);
        }
        drop(allocation);
        Err(WindowsTransportError::PrincipalProofUnavailable)
    }
}

fn last_error_code() -> u32 {
    // SAFETY: GetLastError has no preconditions and reads thread-local error state.
    unsafe { GetLastError() }
}

fn last_error() -> WindowsTransportError {
    WindowsTransportError::Win32(last_error_code())
}

fn token_user_sid(token: HANDLE) -> Result<WindowsUserSid, WindowsTransportError> {
    let mut required = 0_u32;
    // SAFETY: null buffer with zero length is the documented sizing call.
    let first = unsafe { GetTokenInformation(token, TokenUser, null_mut(), 0, &mut required) };
    if first != 0
        || last_error_code() != ERROR_INSUFFICIENT_BUFFER
        || required < size_of::<TOKEN_USER>() as u32
        || required > MAX_TOKEN_USER_BYTES
    {
        return Err(WindowsTransportError::PrincipalProofUnavailable);
    }

    let words = (required as usize).div_ceil(size_of::<usize>());
    let mut buffer = vec![0_usize; words];
    // SAFETY: buffer is aligned and has at least required writable bytes.
    if unsafe {
        GetTokenInformation(
            token,
            TokenUser,
            buffer.as_mut_ptr().cast(),
            required,
            &mut required,
        )
    } == 0
    {
        return Err(last_error());
    }
    // SAFETY: successful TokenUser query wrote a TOKEN_USER at the beginning of the buffer.
    let token_user = unsafe { &*buffer.as_ptr().cast::<TOKEN_USER>() };
    WindowsUserSid::copy_from(token_user.User.Sid)
}

pub(crate) fn current_process_user_sid() -> Result<WindowsUserSid, WindowsTransportError> {
    let mut token: HANDLE = null_mut();
    // SAFETY: GetCurrentProcess returns a pseudo-handle valid for OpenProcessToken.
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
        return Err(last_error());
    }
    if token.is_null() {
        return Err(WindowsTransportError::PrincipalProofUnavailable);
    }
    let token = OwnedHandle(token);
    token_user_sid(token.0)
}

fn current_thread_user_sid() -> Result<WindowsUserSid, WindowsTransportError> {
    let mut token: HANDLE = null_mut();
    // SAFETY: after named-pipe impersonation, the current thread has an impersonation token.
    if unsafe { OpenThreadToken(GetCurrentThread(), TOKEN_QUERY, 1, &mut token) } == 0 {
        return Err(last_error());
    }
    if token.is_null() {
        return Err(WindowsTransportError::PrincipalProofUnavailable);
    }
    let token = OwnedHandle(token);
    token_user_sid(token.0)
}

pub(crate) fn require_same_user_named_pipe_client(
    pipe: HANDLE,
    expected: &WindowsUserSid,
) -> Result<(), WindowsTransportError> {
    // SAFETY: pipe is a connected named-pipe server handle owned by the caller.
    if unsafe { ImpersonateNamedPipeClient(pipe) } == 0 {
        return Err(last_error());
    }
    let result = current_thread_user_sid().and_then(|observed| {
        if expected.matches(observed.as_psid()) {
            Ok(())
        } else {
            Err(WindowsTransportError::PrincipalDenied)
        }
    });
    // SAFETY: the current thread is impersonating only because the call above succeeded.
    if unsafe { RevertToSelf() } == 0 {
        return Err(WindowsTransportError::ImpersonationRevertFailed);
    }
    result
}

pub(crate) fn validate_pipe_owner_and_dacl(
    pipe: HANDLE,
    expected: &WindowsUserSid,
) -> Result<(), WindowsTransportError> {
    let mut owner: PSID = null_mut();
    let mut dacl = null_mut();
    let mut descriptor = null_mut();
    // SAFETY: all output pointers are valid for GetSecurityInfo and pipe is an open handle.
    let status = unsafe {
        GetSecurityInfo(
            pipe,
            SE_KERNEL_OBJECT,
            OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
            &mut owner,
            null_mut(),
            &mut dacl,
            null_mut(),
            &mut descriptor,
        )
    };
    if status != ERROR_SUCCESS {
        return Err(WindowsTransportError::Win32(status));
    }
    if descriptor.is_null() {
        return Err(WindowsTransportError::EffectiveSecurityMismatch);
    }
    let _allocation = LocalAllocation(descriptor);
    if !expected.matches(owner) || dacl.is_null() {
        return Err(WindowsTransportError::EffectiveSecurityMismatch);
    }

    let mut control = 0_u16;
    let mut revision = 0_u32;
    // SAFETY: descriptor is a valid security descriptor returned by GetSecurityInfo.
    if unsafe { GetSecurityDescriptorControl(descriptor, &mut control, &mut revision) } == 0
        || control & SE_DACL_PROTECTED == 0
    {
        return Err(WindowsTransportError::EffectiveSecurityMismatch);
    }

    // SAFETY: dacl is non-null and belongs to descriptor for this function's lifetime.
    let ace_count = unsafe { (*dacl).AceCount };
    if ace_count != 1 {
        return Err(WindowsTransportError::EffectiveSecurityMismatch);
    }
    let mut ace_raw: *mut c_void = null_mut();
    // SAFETY: index 0 is in bounds because AceCount is exactly one.
    if unsafe { GetAce(dacl, 0, &mut ace_raw) } == 0 || ace_raw.is_null() {
        return Err(WindowsTransportError::EffectiveSecurityMismatch);
    }
    // SAFETY: GetAce returned a valid ACE pointer with at least an ACE_HEADER.
    let header = unsafe { &*ace_raw.cast::<ACE_HEADER>() };
    if header.AceType != ACCESS_ALLOWED_ACE_TYPE_VALUE
        || (header.AceSize as usize) < size_of::<ACCESS_ALLOWED_ACE>()
    {
        return Err(WindowsTransportError::EffectiveSecurityMismatch);
    }
    // SAFETY: header type/size prove the layout needed for ACCESS_ALLOWED_ACE.
    let ace = unsafe { &*ace_raw.cast::<ACCESS_ALLOWED_ACE>() };
    let sid = (&raw const ace.SidStart).cast_mut().cast::<c_void>();
    if ace.Mask != GENERIC_ALL || !expected.matches(sid) {
        return Err(WindowsTransportError::EffectiveSecurityMismatch);
    }
    Ok(())
}
