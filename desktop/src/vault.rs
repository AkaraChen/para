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
pub struct FrontmatterField {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenTab {
    pub path: PathBuf,
    pub title: String,
    pub content: String,
    pub frontmatter: Vec<FrontmatterField>,
}

impl OpenTab {
    /// Body plus YAML fields, so the filing assistant still sees `due` / `standard`.
    pub fn excerpt(&self) -> String {
        if self.frontmatter.is_empty() {
            return self.content.clone();
        }
        let meta = self
            .frontmatter
            .iter()
            .map(|field| format!("{}: {}", field.key, field.value))
            .collect::<Vec<_>>()
            .join("\n");
        format!("{}\n\n{meta}", self.content)
    }
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

const PARA_DIR_NAMES: &[&str] = &["Projects", "Areas", "Resources", "Archives"];

const INBOX_SEED: &str = "\
---
updated: 
---

# Inbox

Dump first. File later.
";

const INDEX_SEED: &str = "\
---
title: para
horizon: 
---

# para

Personal PARA store.

| Bucket | Question |
| --- | --- |
| Project | Finish line + outcome? |
| Area | Standard to keep? |
| Resource | Topic to keep? |
| Archive | Inactive but searchable? |

Capture into [INBOX.md](INBOX.md). File with the `para` CLI.
";

/// Home-directory PARA vault (`~/para` on Unix, `%USERPROFILE%\para` on Windows).
pub fn default_user_vault(home: &Path) -> PathBuf {
    home.join("para")
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
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

    if let Some(home) = home_dir() {
        let root = default_user_vault(&home);
        let _ = ensure_vault(&root);
        return root;
    }

    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Create the PARA folders and starter markdown a first-run vault needs.
///
/// Existing files are never overwritten. Missing `Projects` / `Areas` /
/// `Resources` / `Archives`, `INBOX.md`, and `INDEX.md` are created.
pub fn ensure_vault(root: &Path) -> std::io::Result<EnsureReport> {
    fs::create_dir_all(root)?;

    let mut created_dirs = Vec::new();
    let mut created_files = Vec::new();

    for name in PARA_DIR_NAMES {
        let dir = root.join(name);
        if !dir.is_dir() {
            fs::create_dir_all(&dir)?;
            created_dirs.push((*name).to_string());
        }
    }

    if !has_inbox(root) {
        fs::write(root.join("INBOX.md"), INBOX_SEED)?;
        created_files.push("INBOX.md".to_string());
    }
    if !has_index(root) {
        fs::write(root.join("INDEX.md"), INDEX_SEED)?;
        created_files.push("INDEX.md".to_string());
    }

    Ok(EnsureReport {
        created_dirs,
        created_files,
    })
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EnsureReport {
    pub created_dirs: Vec<String>,
    pub created_files: Vec<String>,
}

fn has_inbox(root: &Path) -> bool {
    root.join("INBOX.md").is_file() || root.join("Inbox.md").is_file()
}

fn has_index(root: &Path) -> bool {
    root.join("INDEX.md").is_file() || root.join("Index.md").is_file()
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
    let mut raw = String::from_utf8_lossy(slice).into_owned();
    if truncated {
        raw.push_str("\n\n> Preview truncated after 1 MB.\n");
    }

    let (frontmatter, content) = split_frontmatter(&raw);
    let title = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("untitled.md")
        .to_string();

    Ok(OpenTab {
        path: path.to_path_buf(),
        title,
        content,
        frontmatter,
    })
}

/// Split a leading YAML document (`---` … `---`) from the markdown body.
pub fn split_frontmatter(raw: &str) -> (Vec<FrontmatterField>, String) {
    let text = raw.strip_prefix('\u{feff}').unwrap_or(raw);
    let mut lines = text.lines();
    let Some(first) = lines.next() else {
        return (Vec::new(), String::new());
    };
    if !is_frontmatter_fence(first) {
        return (Vec::new(), raw.to_string());
    }

    let mut yaml_lines = Vec::new();
    let mut closed = false;
    for line in lines.by_ref() {
        if is_frontmatter_fence(line) {
            closed = true;
            break;
        }
        yaml_lines.push(line);
    }
    if !closed {
        return (Vec::new(), raw.to_string());
    }

    let body = lines.collect::<Vec<_>>().join("\n");
    let body = body.trim_start_matches(['\r', '\n']).to_string();
    (parse_frontmatter_fields(&yaml_lines), body)
}

fn is_frontmatter_fence(line: &str) -> bool {
    matches!(line.trim(), "---" | "...")
}

fn parse_frontmatter_fields(lines: &[&str]) -> Vec<FrontmatterField> {
    let mut fields = Vec::new();
    let mut pending_key: Option<String> = None;
    let mut pending_block: Option<BlockStyle> = None;
    let mut pending_values: Vec<String> = Vec::new();
    let mut block_indent: Option<usize> = None;

    let flush = |fields: &mut Vec<FrontmatterField>,
                 key: &mut Option<String>,
                 block: &mut Option<BlockStyle>,
                 values: &mut Vec<String>,
                 indent: &mut Option<usize>| {
        let Some(key) = key.take() else {
            values.clear();
            *block = None;
            *indent = None;
            return;
        };
        let value = match block.take() {
            Some(BlockStyle::Folded) => values
                .iter()
                .map(|line| line.trim())
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>()
                .join(" "),
            Some(BlockStyle::Literal) => values.join("\n"),
            None if values.len() > 1 => values.join(", "),
            None => values.first().cloned().unwrap_or_default(),
        };
        fields.push(FrontmatterField { key, value });
        values.clear();
        *indent = None;
    };

    for line in lines {
        if line.trim().is_empty() {
            if pending_block.is_some() {
                pending_values.push(String::new());
            }
            continue;
        }
        if line.trim_start().starts_with('#') && indent_width(line) == 0 {
            continue;
        }

        let indent = indent_width(line);
        let trimmed = line.trim();

        if indent == 0 {
            if let Some((key, raw_value)) = split_yaml_key_value(trimmed) {
                flush(
                    &mut fields,
                    &mut pending_key,
                    &mut pending_block,
                    &mut pending_values,
                    &mut block_indent,
                );
                match parse_block_style(&raw_value) {
                    Some(style) => {
                        pending_key = Some(key);
                        pending_block = Some(style);
                    }
                    None if raw_value.is_empty() => {
                        pending_key = Some(key);
                    }
                    None => {
                        fields.push(FrontmatterField {
                            key,
                            value: unquote_yaml(&raw_value),
                        });
                    }
                }
                continue;
            }
            flush(
                &mut fields,
                &mut pending_key,
                &mut pending_block,
                &mut pending_values,
                &mut block_indent,
            );
            continue;
        }

        if pending_key.is_none() {
            continue;
        }

        if let Some(style) = pending_block {
            let content = match block_indent {
                Some(min) if indent >= min => {
                    line.get(min..).unwrap_or(trimmed).trim_end().to_string()
                }
                None => {
                    block_indent = Some(indent);
                    trimmed.to_string()
                }
                Some(_) => trimmed.to_string(),
            };
            if style == BlockStyle::Literal || !content.is_empty() {
                pending_values.push(content);
            }
            continue;
        }

        if let Some(item) = trimmed.strip_prefix("- ") {
            pending_values.push(unquote_yaml(item.trim()));
            continue;
        }
        if trimmed == "-" {
            pending_values.push(String::new());
            continue;
        }
        if let Some((key, raw_value)) = split_yaml_key_value(trimmed) {
            if let Some(parent) = pending_key.as_deref() {
                fields.push(FrontmatterField {
                    key: format!("{parent}.{key}"),
                    value: unquote_yaml(&raw_value),
                });
            }
            continue;
        }
        pending_values.push(unquote_yaml(trimmed));
    }

    flush(
        &mut fields,
        &mut pending_key,
        &mut pending_block,
        &mut pending_values,
        &mut block_indent,
    );
    fields
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BlockStyle {
    Literal,
    Folded,
}

fn parse_block_style(value: &str) -> Option<BlockStyle> {
    match value.trim() {
        "|" | "|-" | "|+" => Some(BlockStyle::Literal),
        ">" | ">-" | ">+" => Some(BlockStyle::Folded),
        _ => None,
    }
}

fn split_yaml_key_value(line: &str) -> Option<(String, String)> {
    let (key, value) = line.split_once(':')?;
    let key = unquote_yaml(key.trim());
    if key.is_empty() || key.starts_with('-') {
        return None;
    }
    Some((key, value.trim().to_string()))
}

fn unquote_yaml(value: &str) -> String {
    let value = value.trim();
    let bytes = value.as_bytes();
    if bytes.len() >= 2
        && ((bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\''))
    {
        return value[1..value.len() - 1].to_string();
    }
    value.to_string()
}

fn indent_width(line: &str) -> usize {
    line.chars()
        .take_while(|ch| *ch == ' ' || *ch == '\t')
        .count()
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

    #[test]
    fn default_user_vault_is_home_para() {
        assert_eq!(
            default_user_vault(Path::new("/home/ada")),
            PathBuf::from("/home/ada/para")
        );
    }

    #[test]
    fn ensure_vault_creates_missing_folders_and_starter_markdown() {
        let root = temp_dir();
        let report = ensure_vault(&root).expect("ensure empty vault");

        assert_eq!(
            report.created_dirs,
            vec!["Projects", "Areas", "Resources", "Archives"]
        );
        assert_eq!(report.created_files, vec!["INBOX.md", "INDEX.md"]);

        for name in PARA_DIR_NAMES {
            assert!(root.join(name).is_dir(), "missing {name}");
        }
        assert!(root.join("INBOX.md").is_file());
        assert!(root.join("INDEX.md").is_file());
        let inbox = fs::read_to_string(root.join("INBOX.md")).unwrap();
        assert!(inbox.contains("# Inbox"));
        assert!(inbox.contains("Dump first"));
        let index = fs::read_to_string(root.join("INDEX.md")).unwrap();
        assert!(index.contains("# para"));
        assert!(looks_like_vault(&root));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn ensure_vault_does_not_overwrite_existing_notes() {
        let root = temp_dir();
        fs::create_dir_all(root.join("Projects")).unwrap();
        fs::write(root.join("INBOX.md"), "keep my captures\n").unwrap();
        fs::write(root.join("Projects/ship.md"), "# ship\n").unwrap();

        let report = ensure_vault(&root).expect("fill missing pieces");
        assert_eq!(report.created_dirs, vec!["Areas", "Resources", "Archives"]);
        assert_eq!(report.created_files, vec!["INDEX.md"]);
        assert_eq!(
            fs::read_to_string(root.join("INBOX.md")).unwrap(),
            "keep my captures\n"
        );
        assert_eq!(
            fs::read_to_string(root.join("Projects/ship.md")).unwrap(),
            "# ship\n"
        );
        assert!(root.join("INDEX.md").is_file());
        assert!(root.join("Areas").is_dir());

        let again = ensure_vault(&root).expect("idempotent");
        assert!(again.created_dirs.is_empty());
        assert!(again.created_files.is_empty());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn ensure_vault_treats_inbox_alias_as_present() {
        let root = temp_dir();
        fs::write(root.join("Inbox.md"), "already here\n").unwrap();
        fs::write(root.join("Index.md"), "overview\n").unwrap();

        let report = ensure_vault(&root).expect("aliases count");
        assert!(report.created_files.is_empty());
        assert!(!root.join("INBOX.md").exists());
        assert!(!root.join("INDEX.md").exists());
        assert_eq!(
            fs::read_to_string(root.join("Inbox.md")).unwrap(),
            "already here\n"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn split_frontmatter_reads_scalar_fields() {
        let raw = "---\nid: ship-para-desktop\nstatus: active\noutcome: A GPUI app\n---\n\n# Ship para desktop\n";
        let (fields, body) = split_frontmatter(raw);
        assert_eq!(
            fields
                .iter()
                .map(|field| (field.key.as_str(), field.value.as_str()))
                .collect::<Vec<_>>(),
            vec![
                ("id", "ship-para-desktop"),
                ("status", "active"),
                ("outcome", "A GPUI app"),
            ]
        );
        assert_eq!(body, "# Ship para desktop");
    }

    #[test]
    fn split_frontmatter_keeps_body_when_fence_is_missing() {
        let raw = "# Inbox\n\nDump first.\n";
        let (fields, body) = split_frontmatter(raw);
        assert!(fields.is_empty());
        assert_eq!(body, raw);
    }

    #[test]
    fn split_frontmatter_ignores_unclosed_fence() {
        let raw = "---\nid: broken\n# still markdown\n";
        let (fields, body) = split_frontmatter(raw);
        assert!(fields.is_empty());
        assert_eq!(body, raw);
    }

    #[test]
    fn split_frontmatter_reads_lists_and_blocks() {
        let raw =
            "---\ntags:\n  - para\n  - desktop\nnotes: |\n  line one\n  line two\n---\nbody\n";
        let (fields, body) = split_frontmatter(raw);
        assert_eq!(fields[0].key, "tags");
        assert_eq!(fields[0].value, "para, desktop");
        assert_eq!(fields[1].key, "notes");
        assert_eq!(fields[1].value, "line one\nline two");
        assert_eq!(body, "body");
    }

    #[test]
    fn load_markdown_strips_example_frontmatter() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("example-vault/Projects/ship-para-desktop.md");
        let tab = load_markdown(&path).expect("example note");
        assert_eq!(
            tab.frontmatter
                .iter()
                .map(|field| field.key.as_str())
                .collect::<Vec<_>>(),
            vec!["id", "status", "due", "outcome", "area"]
        );
        assert!(tab.content.starts_with("# Ship para desktop"));
        assert!(!tab.content.starts_with("---"));
        assert!(tab.excerpt().contains("due: 2026-09-15"));
    }
}
