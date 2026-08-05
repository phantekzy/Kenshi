use crate::tree::DirNode;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Category {
    BuildArtifact,
    PackageCache,
    VcsMetadata,
    BrowserCache,
    TempFiles,
    Logs,
}

impl Category {
    pub fn label(&self) -> &'static str {
        match self {
            Category::BuildArtifact => "Build artifacts",
            Category::PackageCache => "Package cache",
            Category::VcsMetadata => "VCS metadata",
            Category::BrowserCache => "Browser cache",
            Category::TempFiles => "Temporary files",
            Category::Logs => "Log files",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Category::BuildArtifact => "Compiled output, safe to delete and rebuild",
            Category::PackageCache => "Downloaded dependency cache, safe to delete and re-fetch",
            Category::VcsMetadata => "Version control history/metadata, do NOT delete casually",
            Category::BrowserCache => "Browser-cached assets, safe to clear",
            Category::TempFiles => "Temporary/scratch files",
            Category::Logs => "Log output, usually safe to delete",
        }
    }
}

/// (directory name, category) — exact, case-insensitive match against a
/// directory's own name. Order doesn't matter; first match wins.
const PATTERNS: &[(&str, Category)] = &[
    ("target", Category::BuildArtifact),      // Rust
    ("build", Category::BuildArtifact),       // generic / Gradle / CMake
    ("dist", Category::BuildArtifact),        // JS bundlers
    ("bin", Category::BuildArtifact),
    ("obj", Category::BuildArtifact),         // .NET
    (".gradle", Category::BuildArtifact),
    ("node_modules", Category::PackageCache), // npm/yarn
    (".cargo", Category::PackageCache),       // Rust registry cache
    (".m2", Category::PackageCache),          // Maven
    ("__pycache__", Category::PackageCache),
    (".venv", Category::PackageCache),
    ("venv", Category::PackageCache),
    (".npm", Category::PackageCache),
    (".git", Category::VcsMetadata),
    (".cache", Category::BrowserCache),
    ("cachestorage", Category::BrowserCache),
    ("tmp", Category::TempFiles),
    ("temp", Category::TempFiles),
    ("logs", Category::Logs),
];

pub struct CleanupFinding {
    pub name: String,
    pub path: std::path::PathBuf,
    pub category: Category,
    pub size: u64,
    pub file_count: u64,
}

fn match_category(dir_name: &str) -> Option<Category> {
    let lower = dir_name.to_lowercase();
    PATTERNS
        .iter()
        .find(|(pattern, _)| *pattern == lower)
        .map(|(_, cat)| *cat)
}

/// Walks the tree and collects every directory matching a known pattern.
/// Once a directory matches, we don't recurse further into it — a `target`
/// folder full of nested build junk gets one finding, not dozens.
pub fn scan(root: &DirNode) -> Vec<CleanupFinding> {
    let mut findings = Vec::new();
    collect(root, &mut findings);
    findings
}

fn collect(node: &DirNode, out: &mut Vec<CleanupFinding>) {
    if !node.is_dir {
        return;
    }
    if let Some(category) = match_category(&node.name) {
        out.push(CleanupFinding {
            name: node.name.clone(),
            path: node.path.clone(),
            category,
            size: node.size,
            file_count: node.file_count,
        });
        return; // don't descend into a directory we've already classified
    }
    for child in &node.children {
        collect(child, out);
    }
}

/// Total reclaimable bytes across all findings, excluding VCS metadata
/// (which shouldn't be casually deleted, so it doesn't count as "reclaimable").
pub fn reclaimable_bytes(findings: &[CleanupFinding]) -> u64 {
    findings
        .iter()
        .filter(|f| f.category != Category::VcsMetadata)
        .map(|f| f.size)
        .sum()
}