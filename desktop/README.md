# para desktop

A [GPUI](https://www.gpui.rs/) + [gpui-component](https://longbridge.github.io/gpui-component/) preview for a PARA vault.

```text
┌────────────┬──────────────────────────┬─────────────────┐
│ Vault tree │ Markdown tabs (read-only)│ Agent chat      │
└────────────┴──────────────────────────┴─────────────────┘
```

- **Left:** file tree of `INBOX.md`, `INDEX.md`, `Projects/`, `Areas/`, `Resources/`, `Archives/`
- **Center:** multi-tab markdown preview (`TextView`, selectable, not editable)
- **Right:** local PARA filing assistant (classifies notes, review checklist, CLI sketches)

The preview is read-only. Use the `para` CLI (or any editor) to capture and file. On first launch the app will create `~/para` if needed, plus any missing PARA folders and starter `INBOX.md` / `INDEX.md`. Existing files are never overwritten.

## Requirements

- Rust 1.98+ (pinned in `rust-toolchain.toml`)
- Linux, macOS, or Windows with the [gpui-component system deps](https://longbridge.github.io/gpui-component/docs/installation)
- On Debian/Ubuntu: `libxkbcommon-dev libwayland-dev libvulkan-dev libssl-dev libfontconfig-dev libfreetype-dev`

## Run

From this directory:

```bash
cargo run
```

That opens `~/para` (created on first run with `INBOX.md`, `INDEX.md`, `Projects/`, `Areas/`, `Resources/`, and `Archives/` if they are missing).

```bash
cargo run -- example-vault
cargo run -- /path/to/vault
PARA_VAULT=~/.para cargo run
```

Resolution order: CLI path → `PARA_VAULT` → `~/para` (ensured) → cwd.

## Tests

Vault scanning and the filing assistant are plain Rust and do not need a display:

```bash
cargo test
```
