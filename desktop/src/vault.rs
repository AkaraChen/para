use std::fs;
use std::path::{Path, PathBuf};

use gpui_component::tree::TreeItem;

const SKIP_DIR_NAMES: &[&str] = &[
    ".git",
    ".hg",
    ".svn",
    ".idea",
    ".vscode",
    ".cursor",
    "node_modules",
    "target",
    "dist",
    "build",
];

const MAX_PREVIEW_BYTES: usize = 1_000_000;
const MAX_DEPTH: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenTab {
    pub path: PathBuf,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParaKind {
    Inbox,
    Index,
    Project,
    Area,
    Resource,
    Archive,
    Note,
}

impl ParaKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Inbox => "Inbox",
            Self::Index => "Index",
            Self::Project => "Project",
            Self::Area => "Area",
            Self::Resource => "Resource",
            Self::Archive => "Archive",
            Self::Note => "Note",
        }
    }
}

pub fn resolve_root() -> PathBuf {
    if let Some(arg) = std::env::args().nth(1) {
        return PathBuf::from(arg);
    }
    if let Ok(from_env) = std::env::var("PARA_VAULT") {
        if !from_env.is_empty() {
            return PathBuf::from(from_env);
        }
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if looks_like_vault(&cwd) {
        return cwd;
    }

    if let Some(home) = std::env::var_os("HOME") {
        let global = PathBuf::from(home).join(".para");
        if looks_like_vault(&global) {
            return global;
        }
    }

    let example = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("example-vault");
    if example.is_dir() {
        return example;
    }

    cwd
}

pub fn looks_like_vault(path: &Path) -> bool {
    path.join("INBOX.md").is_file()
        || path.join("INDEX.md").is_file()
        || path.join("Projects").is_dir()
        || path.join("Areas").is_dir()
        || path.join("Resources").is_dir()
        || path.join("Archives").is_dir()
}

pub fn is_markdown(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()).map(|ext| ext.to_ascii_lowercase()),
        Some(ext) if ext == "md" || ext == "markdown" || ext == "mdown"
    )
}

pub fn classify_path(root: &Path, path: &Path) -> ParaKind {
    let relative = path.strip_prefix(root).unwrap_or(path);
    let first = relative
        .components()
        .next()
        .and_then(|component| component.as_os_str().to_str())
        .unwrap_or("");

    match first {
        "INBOX.md" | "Inbox.md" => ParaKind::Inbox,
        "INDEX.md" | "Index.md" => ParaKind::Index,
        "Projects" => ParaKind::Project,
        "Areas" => ParaKind::Area,
        "Resources" => ParaKind::Resource,
        "Archives" => ParaKind::Archive,
        _ => ParaKind::Note,
    }
}

pub fn load_markdown(path: &Path) -> Result<OpenTab, String> {
    let raw = fs::read(path).map_err(|err| format!("Could not read {}: {err}", path.display()))?;
    let truncated = raw.len() > MAX_PREVIEW_BYTES;
    let slice = if truncated {
        &raw[..MAX_PREVIEW_BYTES]
    } else {
        &raw
    };
    let mut content = String::from_utf8_lossy(slice).into_owned();
    if truncated {
        content.push_str("\n\n> Preview truncated after 1 MB.\n");
    }

    let title = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("untitled.md")
        .to_string();

    Ok(OpenTab {
        path: path.to_path_buf(),
        title,
        content,
    })
}

pub fn default_open_path(root: &Path) -> Option<PathBuf> {
    for name in ["INBOX.md", "INDEX.md"] {
        let candidate = root.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

pub fn scan_tree(root: &Path) -> Vec<TreeItem> {
    scan_dir(root, 0)
}

pub fn vault_file_list(root: &Path) -> Vec<String> {
    let mut files = Vec::new();
    collect_markdown(root, root, 0, &mut files);
    files.sort();
    files
}

fn collect_markdown(root: &Path, dir: &Path, depth: usize, out: &mut Vec<String>) {
    if depth > MAX_DEPTH {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = file_name(&path);
        if path.is_dir() {
            if should_skip_dir(&name) {
                continue;
            }
            collect_markdown(root, &path, depth + 1, out);
        } else if is_markdown(&path) {
            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            out.push(relative);
        }
    }
}

fn scan_dir(dir: &Path, depth: usize) -> Vec<TreeItem> {
    if depth > MAX_DEPTH {
        return Vec::new();
    }

    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut rows: Vec<(u8, String, TreeItem)> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let name = file_name(&path);
        if name.is_empty() {
            continue;
        }

        if path.is_dir() {
            if should_skip_dir(&name) {
                continue;
            }
            let children = scan_dir(&path, depth + 1);
            if children.is_empty() && !is_para_dir(&name) {
                continue;
            }
            let item = TreeItem::new(path.to_string_lossy().to_string(), name.clone())
                .expanded(depth == 0 && is_para_dir(&name))
                .children(children);
            rows.push((sort_rank(&name, true), name, item));
        } else if is_markdown(&path) {
            let item = TreeItem::new(path.to_string_lossy().to_string(), name.clone());
            rows.push((sort_rank(&name, false), name, item));
        }
    }

    rows.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then_with(|| a.1.to_lowercase().cmp(&b.1.to_lowercase()))
    });
    rows.into_iter().map(|(_, _, item)| item).collect()
}

fn should_skip_dir(name: &str) -> bool {
    name.starts_with('.') || SKIP_DIR_NAMES.iter().any(|skip| *skip == name)
}

fn is_para_dir(name: &str) -> bool {
    matches!(name, "Projects" | "Areas" | "Resources" | "Archives")
}

fn sort_rank(name: &str, is_dir: bool) -> u8 {
    match name {
        "INBOX.md" | "Inbox.md" => 0,
        "INDEX.md" | "Index.md" => 1,
        "Projects" => 2,
        "Areas" => 3,
        "Resources" => 4,
        "Archives" => 5,
        _ if is_dir => 6,
        _ => 7,
    }
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("para-vault-test-{nanos}"));
        fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    #[test]
    fn markdown_extensions() {
        assert!(is_markdown(Path::new("INBOX.md")));
        assert!(is_markdown(Path::new("note.MARKDOWN")));
        assert!(!is_markdown(Path::new("reviews.ndjson")));
    }

    #[test]
    fn classifies_para_paths() {
        let root = Path::new("/notes");
        assert_eq!(
            classify_path(root, Path::new("/notes/INBOX.md")),
            ParaKind::Inbox
        );
        assert_eq!(
            classify_path(root, Path::new("/notes/Projects/ship.md")),
            ParaKind::Project
        );
        assert_eq!(
            classify_path(root, Path::new("/notes/Resources/para-method.md")),
            ParaKind::Resource
        );
        assert_eq!(
            classify_path(root, Path::new("/notes/scratch.md")),
            ParaKind::Note
        );
    }

    #[test]
    fn scan_keeps_para_order_and_skips_noise() {
        let root = temp_dir();
        fs::write(root.join("INDEX.md"), "# index\n").unwrap();
        fs::write(root.join("INBOX.md"), "# inbox\n").unwrap();
        fs::create_dir_all(root.join("Projects")).unwrap();
        fs::write(root.join("Projects/ship.md"), "# ship\n").unwrap();
        fs::create_dir_all(root.join("target")).unwrap();
        fs::write(root.join("target/out.md"), "# skip\n").unwrap();
        fs::create_dir_all(root.join("empty-notes")).unwrap();

        let items = scan_tree(&root);
        let labels: Vec<String> = items.iter().map(|item| item.label.to_string()).collect();
        assert_eq!(labels, vec!["INBOX.md", "INDEX.md", "Projects"]);

        let files = vault_file_list(&root);
        assert_eq!(
            files,
            vec![
                "INBOX.md".to_string(),
                "INDEX.md".to_string(),
                "Projects/ship.md".to_string()
            ]
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn looks_like_vault_detects_inbox() {
        let root = temp_dir();
        assert!(!looks_like_vault(&root));
        fs::write(root.join("INBOX.md"), "").unwrap();
        assert!(looks_like_vault(&root));
        let _ = fs::remove_dir_all(root);
    }
}
