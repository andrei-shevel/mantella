//! Content-based file identity, so persisted state (pins, reading position,
//! bookmarks) survives a file being renamed or moved on disk.

use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

/// Bytes read from each end of the file when computing an identity hash.
pub const CHUNK_SIZE: u64 = 64 * 1024;

/// A stable, content-based id for a file: `"{16-hex-chars}-{size}"`, from a
/// partial SHA-256 (first `CHUNK_SIZE` + last `CHUNK_SIZE` bytes; the whole
/// file if it's smaller than 2×`CHUNK_SIZE`, to avoid double-hashing
/// overlapping bytes) plus size. Two files with identical size and identical
/// leading/trailing bytes share an id by design: this is how rename/move
/// survives, and duplicate content is intentionally treated as one identity.
pub fn compute_id_with_size(path: &Path, size: u64) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();

    if size <= CHUNK_SIZE * 2 {
        io::copy(&mut file, &mut hasher)?;
    } else {
        let mut buf = vec![0u8; CHUNK_SIZE as usize];
        file.read_exact(&mut buf)?;
        hasher.update(&buf);
        file.seek(SeekFrom::End(-(CHUNK_SIZE as i64)))?;
        file.read_exact(&mut buf)?;
        hasher.update(&buf);
    }

    let digest = hasher.finalize();
    let prefix = u64::from_be_bytes(digest[..8].try_into().unwrap());
    Ok(format!("{prefix:016x}-{size}"))
}

/// `metadata.modified()` as epoch seconds.
pub fn mtime_secs(metadata: &std::fs::Metadata) -> Option<u64> {
    metadata
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs())
}

/// In-memory, session-only memo of path -> (size, mtime, id), so repeated
/// scans (the watcher rescans the whole tree on every debounced fs event)
/// don't rehash files that haven't changed. Not persisted; empty on launch.
pub struct IdentityCache {
    entries: HashMap<PathBuf, (u64, Option<u64>, String)>,
}

impl IdentityCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Returns the cached id if `path`'s size+mtime are unchanged since the
    /// last call; otherwise hashes fresh and updates the cache.
    pub fn resolve(&mut self, path: &Path, size: u64, modified: Option<u64>) -> io::Result<String> {
        if let Some((c_size, c_mtime, id)) = self.entries.get(path) {
            if *c_size == size && *c_mtime == modified {
                return Ok(id.clone());
            }
        }
        let id = compute_id_with_size(path, size)?;
        self.entries
            .insert(path.to_path_buf(), (size, modified, id.clone()));
        Ok(id)
    }
}
