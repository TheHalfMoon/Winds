use std::io;

pub(crate) mod unix;

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

impl From<io::Error> for PosixTransportError {
    fn from(error: io::Error) -> Self {
        Self::Io(error.kind())
    }
}
