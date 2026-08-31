use std::path::PathBuf;

use crate::vault::{classify_path, ParaKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatRole {
    User,
    Assistant,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub text: String,
}

impl ChatMessage {
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: ChatRole::User,
            text: text.into(),
        }
    }

    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            role: ChatRole::Assistant,
            text: text.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AgentContext {
    pub vault_root: PathBuf,
    pub files: Vec<String>,
    pub open_path: Option<PathBuf>,
    pub open_title: Option<String>,
    pub open_excerpt: Option<String>,
}

pub fn welcome(ctx: &AgentContext) -> String {
    let lang = if path_looks_cjk(&ctx.vault_root) {
        Lang::Zh
    } else {
        Lang::En
    };
    welcome_for(lang, ctx)
}

pub fn reply(user: &str, ctx: &AgentContext) -> String {
    let lang = detect_lang(user);
    let lowered = user.to_ascii_lowercase();
    let kind = ctx
        .open_path
        .as_ref()
        .map(|path| classify_path(&ctx.vault_root, path));

    if looks_like_help(&lowered, user) {
        return help(lang);
    }
    if looks_like_review(&lowered, user) {
        return review(lang);
    }
    if looks_like_open_file(&lowered, user) {
        return current_note(lang, ctx, kind);
    }
    if looks_like_classify(&lowered, user) || looks_like_capture(user) {
        return classify(lang, user, ctx, kind);
    }

    default_reply(lang, ctx, kind)
}

#[derive(Clone, Copy)]
enum Lang {
    En,
    Zh,
}

fn detect_lang(text: &str) -> Lang {
    if text.chars().any(is_cjk) {
        Lang::Zh
    } else {
        Lang::En
    }
}

fn is_cjk(ch: char) -> bool {
    matches!(ch,
        '\u{4E00}'..='\u{9FFF}'
        | '\u{3400}'..='\u{4DBF}'
        | '\u{F900}'..='\u{FAFF}'
    )
}

fn path_looks_cjk(path: &std::path::Path) -> bool {
    path.to_string_lossy().chars().any(is_cjk)
}

fn looks_like_help(lowered: &str, raw: &str) -> bool {
    lowered.contains("help")
        || lowered.contains("how")
        || lowered.contains("what is para")
        || raw.contains("怎么")
        || raw.contains("如何")
        || raw.contains("帮助")
        || raw.contains("什么是")
}

fn looks_like_review(lowered: &str, raw: &str) -> bool {
    lowered.contains("review") || raw.contains("回顾") || raw.contains("复盘") || raw.contains("清理 inbox")
}

fn looks_like_open_file(lowered: &str, raw: &str) -> bool {
    lowered.contains("this note")
        || lowered.contains("current")
        || lowered.contains("open file")
        || raw.contains("这篇")
        || raw.contains("当前")
        || raw.contains("打开的")
}

fn looks_like_classify(lowered: &str, raw: &str) -> bool {
    lowered.contains("classif")
        || lowered.contains("where should")
        || lowered.contains("file this")
        || lowered.contains("which bucket")
        || raw.contains("归类")
        || raw.contains("归档")
        || raw.contains("放哪")
        || raw.contains("属于")
}

fn looks_like_capture(user: &str) -> bool {
    user.lines().count() > 2 || user.chars().count() > 80
}

fn welcome_for(lang: Lang, ctx: &AgentContext) -> String {
    match lang {
        Lang::En => format!(
            "I am the local PARA filing assistant for `{}`.\n\n\
            Open a markdown file in the tree, then ask me to classify it, plan a weekly review, or file a raw capture.\n\n\
            Vault files: {}.",
            ctx.vault_root.display(),
            summarize_files(&ctx.files)
        ),
        Lang::Zh => format!(
            "我是这个库 `{}` 的本地 PARA 归档助手。\n\n\
            左边打开一篇笔记，就可以让我归类、做 weekly review，或处理一段 inbox 草稿。\n\n\
            当前库里有：{}。",
            ctx.vault_root.display(),
            summarize_files(&ctx.files)
        ),
    }
}

fn help(lang: Lang) -> String {
    match lang {
        Lang::En => "\
PARA is a filing system, not a writing system.

| Bucket | Question | Lives until |
| --- | --- | --- |
| **Project** | Finish line + concrete outcome? | Outcome ships, then archive |
| **Area** | Standing responsibility with a standard? | The role ends |
| **Resource** | Topic you want to keep? | It stops being useful |
| **Archive** | Inactive but worth keeping? | Forever, for search |

Capture into `INBOX.md` first. On review, give each item one home with the `para` CLI — do not invent a fifth top-level folder.

Try: `classify this`, `review`, or paste a raw note."
            .into(),
        Lang::Zh => "\
PARA 是归档系统，不是写作系统。

| 桶 | 判断 | 放到什么时候 |
| --- | --- | --- |
| **Project** | 有截止日期和具体成果？ | 做完再 archive |
| **Area** | 长期职责、有标准要维持？ | 职责结束 |
| **Resource** | 想长期留着的主题？ | 不再有用再 archive |
| **Archive** | 已停但值得检索？ | 一直留着 |

先写进 `INBOX.md`。回顾时用 `para` CLI 给每条一个归宿，不要自造第五个顶层目录。

可以试：`归类这篇`、`review`，或直接贴一段草稿。"
            .into(),
    }
}

fn review(lang: Lang) -> String {
    match lang {
        Lang::En => "\
**Weekly review**

1. Inbox to zero — each line gets one home.
2. Check project `status` / `due` / `outcome`. Done projects become archives.
3. A project without a due date or outcome is an area in disguise.
4. Log the sweep: `para review append --kind weekly --focus inbox`.

**Monthly:** are area standards still true? Are resources still used?

Use the CLI to move notes. This preview is read-only."
            .into(),
        Lang::Zh => "\
**每周回顾**

1. Inbox 清零，每条只进一个桶。
2. 看项目的 `status` / `due` / `outcome`。做完的项目进 Archives。
3. 没有截止日期或成果的项目，其实是 Area。
4. 记一笔：`para review append --kind weekly --focus inbox`。

**每月：** Area 的标准还成立吗？Resource 还在用吗？

移动笔记请用 `para` CLI。这个预览是只读的。"
            .into(),
    }
}

fn current_note(lang: Lang, ctx: &AgentContext, kind: Option<ParaKind>) -> String {
    let Some(path) = ctx.open_path.as_ref() else {
        return match lang {
            Lang::En => "No markdown tab is open. Pick a file in the tree first.".into(),
            Lang::Zh => "还没有打开笔记。先在左边点一篇 markdown。".into(),
        };
    };

    let kind = kind.unwrap_or(ParaKind::Note);
    let excerpt = excerpt_of(ctx.open_excerpt.as_deref().unwrap_or(""));
    match lang {
        Lang::En => format!(
            "**{}** is filed as a **{}**.\n\nPath: `{}`\n\n{}\n\n{}",
            ctx.open_title.as_deref().unwrap_or("Untitled"),
            kind.label(),
            display_path(&ctx.vault_root, path),
            advice(lang, kind),
            excerpt
        ),
        Lang::Zh => format!(
            "**{}** 现在在 **{}**。\n\n路径：`{}`\n\n{}\n\n{}",
            ctx.open_title.as_deref().unwrap_or("未命名"),
            kind.label(),
            display_path(&ctx.vault_root, path),
            advice(lang, kind),
            excerpt
        ),
    }
}

fn classify(lang: Lang, user: &str, ctx: &AgentContext, open_kind: Option<ParaKind>) -> String {
    let source = if looks_like_capture(user) {
        user
    } else {
        ctx.open_excerpt.as_deref().unwrap_or(user)
    };
    let guessed = guess_bucket(source, open_kind);
    let snippet = first_line(source);

    match lang {
        Lang::En => format!(
            "I would file this as a **{}**.\n\n> {}\n\n{}\n\nCLI sketch:\n```\n{}\n```",
            guessed.label(),
            snippet,
            advice(lang, guessed),
            cli_for(guessed, snippet)
        ),
        Lang::Zh => format!(
            "这条更像 **{}**。\n\n> {}\n\n{}\n\n可以用：\n```\n{}\n```",
            guessed.label(),
            snippet,
            advice(lang, guessed),
            cli_for(guessed, snippet)
        ),
    }
}

fn default_reply(lang: Lang, ctx: &AgentContext, kind: Option<ParaKind>) -> String {
    let open = match (lang, ctx.open_title.as_deref(), kind) {
        (Lang::En, Some(title), Some(kind)) => {
            format!(" Open tab: `{title}` ({})", kind.label())
        }
        (Lang::Zh, Some(title), Some(kind)) => {
            format!(" 当前打开：`{title}`（{}）", kind.label())
        }
        _ => String::new(),
    };

    match lang {
        Lang::En => format!(
            "Ask me to classify the open note, paste a capture to file, or say `review`.{open}\n\nVault files: {}.",
            summarize_files(&ctx.files)
        ),
        Lang::Zh => format!(
            "可以让我归类当前笔记、贴一段草稿，或者说 `review`。{open}\n\n库里的文件：{}。",
            summarize_files(&ctx.files)
        ),
    }
}

fn advice(lang: Lang, kind: ParaKind) -> &'static str {
    match (lang, kind) {
        (Lang::En, ParaKind::Inbox) => {
            "Inbox is a holding area. On the next review, give each bullet one home and clear the body."
        }
        (Lang::Zh, ParaKind::Inbox) => "Inbox 只是中转。下次回顾时每条只进一个桶，然后清空正文。",
        (Lang::En, ParaKind::Index) => {
            "Index is the vault overview. Keep the horizon current; do not dump captures here."
        }
        (Lang::Zh, ParaKind::Index) => "Index 是库总览。保持 horizon 最新，不要把草稿堆在这里。",
        (Lang::En, ParaKind::Project) => {
            "Needs a due date and a one-sentence outcome. When status is `done`, archive it."
        }
        (Lang::Zh, ParaKind::Project) => "项目要有截止日期和一句成果。`done` 之后进 Archives。",
        (Lang::En, ParaKind::Area) => {
            "No finish line. Review the standard; archive only when the responsibility ends."
        }
        (Lang::Zh, ParaKind::Area) => "Area 没有终点。只复盘标准；职责结束再归档。",
        (Lang::En, ParaKind::Resource) => {
            "References have a topic, not an owner or deadline. Archive when you stop using them."
        }
        (Lang::Zh, ParaKind::Resource) => "Resource 是主题资料，没有负责人和截止日期。不用了再归档。",
        (Lang::En, ParaKind::Archive) => {
            "Keep for search. Do not reopen it as a daily working note."
        }
        (Lang::Zh, ParaKind::Archive) => "Archive 留给检索，不要再当日常工作笔记打开。",
        (Lang::En, ParaKind::Note) => {
            "Not under a PARA folder yet. Decide: project, area, resource, or still inbox."
        }
        (Lang::Zh, ParaKind::Note) => "还不在 PARA 目录里。先决定：项目、职责、资料，还是继续留在 inbox。",
    }
}

fn guess_bucket(text: &str, fallback: Option<ParaKind>) -> ParaKind {
    let lowered = text.to_ascii_lowercase();
    let zh_project = text.contains("截止") || text.contains("交付") || text.contains("上线");
    let zh_area = text.contains("每周") || text.contains("职责") || text.contains("保持");
    let zh_resource = text.contains("资料") || text.contains("笔记") || text.contains("文章");
    let zh_archive = text.contains("已完成") || text.contains("归档");

    if lowered.contains("due")
        || lowered.contains("deadline")
        || lowered.contains("ship")
        || lowered.contains("launch")
        || lowered.contains("outcome")
        || zh_project
    {
        return ParaKind::Project;
    }
    if lowered.contains("weekly")
        || lowered.contains("responsibility")
        || lowered.contains("standard")
        || lowered.contains("ongoing")
        || zh_area
    {
        return ParaKind::Area;
    }
    if lowered.contains("article")
        || lowered.contains("book")
        || lowered.contains("reference")
        || lowered.contains("research")
        || zh_resource
    {
        return ParaKind::Resource;
    }
    if lowered.contains("done") || lowered.contains("inactive") || zh_archive {
        return ParaKind::Archive;
    }
    fallback.unwrap_or(ParaKind::Inbox)
}

fn cli_for(kind: ParaKind, snippet: &str) -> String {
    let id = slug(snippet);
    match kind {
        ParaKind::Project => format!(
            "para project create --id {id} --status active --due YYYY-MM-DD --outcome \"…\""
        ),
        ParaKind::Area => {
            format!("para area create --id {id} --status active --standard \"…\"")
        }
        ParaKind::Resource => {
            format!("para resource create --id {id} --topic \"…\" --kind note")
        }
        ParaKind::Archive => {
            format!("para archive create --id {id} --origin project --archived YYYY-MM-DD")
        }
        ParaKind::Inbox | ParaKind::Index | ParaKind::Note => {
            "para inbox write --updated YYYY-MM-DD --body \"…\"".into()
        }
    }
}

fn slug(text: &str) -> String {
    let mut out = String::new();
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if (ch == ' ' || ch == '-' || ch == '_') && !out.ends_with('-') {
            out.push('-');
        }
        if out.len() >= 32 {
            break;
        }
    }
    let slug = out.trim_matches('-').to_string();
    if slug.is_empty() {
        "untitled".into()
    } else {
        slug
    }
}

fn first_line(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with("---"))
        .unwrap_or("untitled")
        .chars()
        .take(120)
        .collect()
}

fn excerpt_of(text: &str) -> String {
    let body = text.trim();
    if body.is_empty() {
        return String::new();
    }
    let short: String = body.chars().take(360).collect();
    if body.chars().count() > 360 {
        format!("{short}…")
    } else {
        short
    }
}

fn summarize_files(files: &[String]) -> String {
    if files.is_empty() {
        return "(empty vault)".into();
    }
    let shown = files.iter().take(8).cloned().collect::<Vec<_>>().join(", ");
    if files.len() > 8 {
        format!("{shown}, +{} more", files.len() - 8)
    } else {
        shown
    }
}

fn display_path(root: &std::path::Path, path: &std::path::Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn ctx() -> AgentContext {
        AgentContext {
            vault_root: PathBuf::from("/notes"),
            files: vec!["INBOX.md".into(), "Projects/ship.md".into()],
            open_path: Some(PathBuf::from("/notes/INBOX.md")),
            open_title: Some("INBOX.md".into()),
            open_excerpt: Some("- Ship the desktop preview by Friday".into()),
        }
    }

    #[test]
    fn classifies_deadline_as_project() {
        let reply = reply("classify: ship the installer by Friday", &ctx());
        assert!(reply.contains("Project"), "{reply}");
        assert!(reply.contains("para project create"), "{reply}");
    }

    #[test]
    fn chinese_review_uses_chinese() {
        let reply = reply("怎么做 weekly review", &ctx());
        assert!(reply.contains("每周") || reply.contains("Inbox"), "{reply}");
    }

    #[test]
    fn open_note_mentions_inbox() {
        let reply = reply("what about this note", &ctx());
        assert!(reply.contains("Inbox"), "{reply}");
    }
}
