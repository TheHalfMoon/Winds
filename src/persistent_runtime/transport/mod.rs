#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::io;

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub(crate) mod unix;
#[cfg(windows)]
pub(crate) mod windows;

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PosixTransportError {
    RuntimeDirectoryNotAbsolute,
    RuntimeDirectoryMissingParent,
    RuntimeDirectorySymlink,
    RuntimeDirectoryPathAlias,
    RuntimeDirectoryNotDirectory,
    RuntimeDirectoryOwnershipMismatch,
    RuntimeDirectoryModeMismatch,
    BindLockUnavailable,
    EndpointPathTooLong,
    EndpointSymlink,
    EndpointCollision,
    EndpointOwnershipMismatch,
    EndpointModeMismatch,
    LiveEndpointCollision,
    StaleEndpointUnproven,
    EndpointIdentityChanged,
    PrincipalDenied,
    PeerCredentialUnavailable,
    EntropyUnavailable,
    EntropyShortRead,
    EntropyInvalid,
    Io(io::ErrorKind),
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
impl From<io::Error> for PosixTransportError {
    fn from(error: io::Error) -> Self {
        Self::Io(error.kind())
    }
}
