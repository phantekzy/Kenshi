use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::SystemTime;

const MAX_PARALLEL_DEPTH: usize = 3;

const MAX_LIVE_THREADS: usize = 64;

static LIVE_THREADS: AtomicUsize = AtomicUsize::new(0);

pub struct DirNode {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub is_dir: bool,
    pub file_count: u64,
    pub readable: bool,
    pub modified: SystemTime,
    pub children: Vec<DirNode>,
}

impl DirNode {
    pub fn scan(path: &Path) -> DirNode {
        Self::scan_at(path, 0)
    }

    fn scan_at(path: &Path, depth: usize) -> DirNode {
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        let meta = fs::symlink_metadata(path);

        let is_symlink = meta.as_ref().map(|m| m.is_symlink()).unwrap_or(false);
        let is_dir = !is_symlink && path.is_dir();
        let own_modified = meta
            .as_ref()
            .ok()
            .and_then(|m| m.modified().ok())
            .unwrap_or(SystemTime::UNIX_EPOCH);

        if is_dir {
            let entries: Vec<PathBuf> = match fs::read_dir(path) {
                Ok(rd) => rd.filter_map(|e| e.ok()).map(|e| e.path()).collect(),
                Err(_) => {
                    return DirNode {
                        name,
                        path: path.to_path_buf(),
                        size: 0,
                        is_dir: true,
                        file_count: 0,
                        readable: false,
                        modified: own_modified,
                        children: vec![],
                    };
                }
            };

            let children = Self::scan_children(&entries, depth);

            let size: u64 = children.iter().map(|c| c.size).sum();
            let file_count: u64 = children
                .iter()
                .map(|c| if c.is_dir { c.file_count } else { 1 })
                .sum();
            // A folder's "modified" is the most recent touch anywhere inside it
            // (or its own timestamp if that's more recent) — the same "when did
            // this last change" signal TreeSize/WizTree show for directories.
            let modified = children
                .iter()
                .map(|c| c.modified)
                .max()
                .unwrap_or(own_modified)
                .max(own_modified);

            let mut children = children;
            children.sort_by(|a, b| b.size.cmp(&a.size));

            DirNode {
                name,
                path: path.to_path_buf(),
                size,
                is_dir: true,
                file_count,
                readable: true,
                modified,
                children,
            }
        } else {
            let size = meta.map(|m| m.len()).unwrap_or(0);
            DirNode {
                name,
                path: path.to_path_buf(),
                size,
                is_dir: false,
                file_count: 1,
                readable: true,
                modified: own_modified,
                children: vec![],
            }
        }
    }

    fn scan_children(entries: &[PathBuf], depth: usize) -> Vec<DirNode> {
        if depth >= MAX_PARALLEL_DEPTH || entries.len() < 2 {
            return entries
                .iter()
                .map(|p| Self::scan_at(p, depth + 1))
                .collect();
        }

        thread::scope(|scope| {
            let handles: Vec<_> = entries
                .iter()
                .map(|p| {
                    let can_spawn = LIVE_THREADS.fetch_add(1, Ordering::SeqCst) < MAX_LIVE_THREADS;
                    if !can_spawn {
                        LIVE_THREADS.fetch_sub(1, Ordering::SeqCst);
                        None
                    } else {
                        Some(scope.spawn(move || {
                            let node = Self::scan_at(p, depth + 1);
                            LIVE_THREADS.fetch_sub(1, Ordering::SeqCst);
                            node
                        }))
                    }
                })
                .collect();

            handles
                .into_iter()
                .zip(entries.iter())
                .map(|(h, p)| match h {
                    Some(handle) => handle
                        .join()
                        .unwrap_or_else(|_| Self::scan_at(p, depth + 1)),
                    None => Self::scan_at(p, depth + 1),
                })
                .collect()
        })
    }

    pub fn sort_by_size(&mut self) {
        self.children.sort_by(|a, b| b.size.cmp(&a.size));
    }

    pub fn sort_by_name(&mut self) {
        self.children
            .sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    }

    pub fn sort_by_modified(&mut self) {
        self.children.sort_by(|a, b| b.modified.cmp(&a.modified));
    }
}
