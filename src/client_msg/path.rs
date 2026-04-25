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
//! also appear in lojix-schema verbs).

use std::path::{Path, PathBuf};

use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

/// Raw OS path bytes. Unix-shaped: a POSIX path is a sequence
/// of bytes terminated by `/`-separated components.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Clone, Debug, PartialEq, Eq, Hash)]
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

#[cfg(test)]
#[cfg(unix)]
mod tests {
    use super::*;

    #[test]
    fn from_path_then_as_path_round_trip() {
        let p = Path::new("/tmp/example/file.txt");
        let wp = WirePath::from_path(p);
        assert_eq!(wp.as_path(), p);
    }

    #[test]
    fn round_trip_preserves_non_utf8_bytes() {
        // POSIX paths can contain arbitrary bytes; ensure the
        // raw-bytes wire form survives a round-trip without
        // string normalisation.
        let bytes = vec![b'/', 0xff, 0xfe, b'/', b'a'];
        let wp = WirePath(bytes.clone());
        let back = WirePath::from_path(wp.as_path());
        assert_eq!(back.0, bytes);
    }
}
