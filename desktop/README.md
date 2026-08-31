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

This app does not write the vault. Use the `para` CLI to capture and file.

## Requirements

- Rust 1.98+ (pinned in `rust-toolchain.toml`)
- Linux, macOS, or Windows with the [gpui-component system deps](https://longbridge.github.io/gpui-component/docs/installation)
- On Debian/Ubuntu: `libxkbcommon-dev libwayland-dev libvulkan-dev libssl-dev libfontconfig-dev libfreetype-dev`

## Run

From this directory:

```bash
cargo run
```

That opens `example-vault/` unless the current directory already looks like a PARA store.

```bash
cargo run -- /path/to/vault
PARA_VAULT=~/.para cargo run
```

Resolution order: CLI path → `PARA_VAULT` → cwd if it has `INBOX.md` / PARA folders → `~/.para` → bundled `example-vault/`.

## Tests

Vault scanning and the filing assistant are plain Rust and do not need a display:

```bash
cargo test
```
