use crate::persistent_runtime::transport::PosixTransportError;
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;

pub(crate) fn current_effective_uid() -> u32 {
    // SAFETY: geteuid has no preconditions and only reads process identity.
    unsafe { libc::geteuid() as u32 }
}

pub(crate) fn require_same_effective_user(
    expected_uid: u32,
    observed_uid: u32,
) -> Result<(), PosixTransportError> {
    if expected_uid == observed_uid {
        Ok(())
    } else {
        Err(PosixTransportError::PrincipalDenied)
    }
}

#[cfg(target_os = "linux")]
pub(crate) fn peer_effective_uid(stream: &UnixStream) -> Result<u32, PosixTransportError> {
    let mut credential = libc::ucred {
        pid: 0,
        uid: 0,
        gid: 0,
    };
    let mut length = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    // SAFETY: the stream owns a valid connected Unix-domain socket descriptor; credential and
    // length point to initialized writable storage of the exact SO_PEERCRED structure size.
    let result = unsafe {
        libc::getsockopt(
            stream.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            (&mut credential as *mut libc::ucred).cast(),
            &mut length,
        )
    };
    if result != 0 || length as usize != std::mem::size_of::<libc::ucred>() {
        return Err(PosixTransportError::PeerCredentialUnavailable);
    }
    Ok(credential.uid)
}

#[cfg(target_os = "macos")]
pub(crate) fn peer_effective_uid(stream: &UnixStream) -> Result<u32, PosixTransportError> {
    let mut effective_uid: libc::uid_t = 0;
    let mut effective_gid: libc::gid_t = 0;
    // SAFETY: the stream owns a valid connected Unix-domain socket descriptor and both output
    // pointers refer to initialized writable uid/gid storage required by getpeereid(3).
    let result =
        unsafe { libc::getpeereid(stream.as_raw_fd(), &mut effective_uid, &mut effective_gid) };
    if result != 0 {
        return Err(PosixTransportError::PeerCredentialUnavailable);
    }
    Ok(effective_uid as u32)
}

pub(crate) fn require_same_user_peer(stream: &UnixStream) -> Result<u32, PosixTransportError> {
    let expected = current_effective_uid();
    let observed = peer_effective_uid(stream)?;
    require_same_effective_user(expected, observed)?;
    Ok(observed)
}
