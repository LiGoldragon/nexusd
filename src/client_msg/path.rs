//! [`WirePath`] — raw OS path bytes for wire transport.
//!
//! Paths on Unix are byte sequences (POSIX), not strings. The
//! daemons run on CriomOS / NixOS; path bytes are deterministic
//! and lossless. We do not string-encode paths on the wire —
//! the bytes ARE the canonical representation. Round-tripping
//! through UTF-8 String would lose non-UTF-8 byte sequences and
//! introduce normalisation hazards for no benefit.
//!
//! When `criome-types` lands, this newtype moves there (paths
//! also appear in lojix-msg verbs).

use std::path::{Path, PathBuf};

/// Raw OS path bytes. Unix-shaped: a POSIX path is a sequence
/// of bytes terminated by `/`-separated components.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WirePath(pub Vec<u8>);

impl WirePath {
    /// Construct from an existing OS path (Unix).
    #[cfg(unix)]
    pub fn from_path(path: &Path) -> Self {
        use std::os::unix::ffi::OsStrExt;
        Self(path.as_os_str().as_bytes().to_vec())
    }

    /// Borrow the bytes as an OS path (Unix).
    #[cfg(unix)]
    pub fn as_path(&self) -> &Path {
        use std::os::unix::ffi::OsStrExt;
        Path::new(std::ffi::OsStr::from_bytes(&self.0))
    }

    /// Convert to an owned [`PathBuf`] (Unix).
    #[cfg(unix)]
    pub fn to_path_buf(&self) -> PathBuf {
        self.as_path().to_path_buf()
    }
}
