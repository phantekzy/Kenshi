use crate::tree::DirNode;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};

/// Cache entries older than this are pruned on startup rather than trusted.
const CACHE_TTL: Duration = Duration::from_secs(60 * 60 * 24 * 14); // 14 days

#[derive(Serialize, Deserialize)]
pub struct CachedNode {
    name: String,
    path: PathBuf,
    size: u64,
    is_dir: bool,
    file_count: u64,
    readable: bool,
    modified_secs: u64,
    children: Vec<CachedNode>,
}

impl From<&DirNode> for CachedNode {
    fn from(n: &DirNode) -> Self {
        CachedNode {
            name: n.name.clone(),
            path: n.path.clone(),
            size: n.size,
            is_dir: n.is_dir,
            file_count: n.file_count,
            readable: n.readable,
            modified_secs: n
                .modified
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            children: n.children.iter().map(CachedNode::from).collect(),
        }
    }
}

impl CachedNode {
    fn into_dir_node(self) -> DirNode {
        DirNode {
            name: self.name,
            path: self.path,
            size: self.size,
            is_dir: self.is_dir,
            file_count: self.file_count,
            readable: self.readable,
            modified: UNIX_EPOCH + Duration::from_secs(self.modified_secs),
            children: self
                .children
                .into_iter()
                .map(CachedNode::into_dir_node)
                .collect(),
        }
    }
}

fn cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("kenshii")
}

fn cache_file_for(scan_path: &Path) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    scan_path.hash(&mut hasher);
    cache_dir().join(format!("{:x}.cache", hasher.finish()))
}

/// Loads a cached tree for this path, but only if the cache file itself
/// isn't older than CACHE_TTL. Returns None on any miss, error, or staleness.
pub fn load(scan_path: &Path) -> Option<DirNode> {
    let file = cache_file_for(scan_path);
    let meta = std::fs::metadata(&file).ok()?;
    let age = meta.modified().ok()?.elapsed().ok()?;
    if age > CACHE_TTL {
        let _ = std::fs::remove_file(&file);
        return None;
    }
    let bytes = std::fs::read(&file).ok()?;
    let cached: CachedNode = bincode::deserialize(&bytes).ok()?;
    Some(cached.into_dir_node())
}

pub fn save(scan_path: &Path, root: &DirNode) -> io::Result<()> {
    let dir = cache_dir();
    std::fs::create_dir_all(&dir)?;
    let cached = CachedNode::from(root);
    let bytes =
        bincode::serialize(&cached).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    std::fs::write(cache_file_for(scan_path), bytes)
}

/// Deletes any cache file untouched for longer than CACHE_TTL.
/// Call once at startup to keep ~/.cache/kenshii/ from accumulating
/// entries for folders you no longer scan.
pub fn prune_stale() {
    let dir = cache_dir();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(meta) = entry.metadata() else { continue };
        let Ok(age) = meta.modified().and_then(|m| {
            m.elapsed()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
        }) else {
            continue;
        };
        if age > CACHE_TTL {
            let _ = std::fs::remove_file(&path);
        }
    }
}