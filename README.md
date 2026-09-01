# para

A [PARA](https://fortelabs.com/blog/para/) notes CLI generated from a [ctxl](https://github.com/AkaraChen/ctxl) schema.

Capture into an inbox, then file each note as a **Project**, **Area**, **Resource**, or **Archive**. The schema in [`context.schema.json`](context.schema.json) is the single source of truth for commands, store layout, and bundled Agent Skills.

## Install

Unix-like (Linux, macOS, FreeBSD):

```bash
curl -fsSL https://raw.githubusercontent.com/AkaraChen/para/main/scripts/install.sh | sh
```

Windows (PowerShell):

```powershell
irm https://raw.githubusercontent.com/AkaraChen/para/main/scripts/install.ps1 | iex
```

The scripts install the latest GitHub Release binary for your OS and architecture and verify the published checksum. Override the destination with `PARA_PREFIX`, or pin a release with `PARA_VERSION=0.1.0`.

`go install` also works if you already have Go 1.24+:

```bash
go install github.com/AkaraChen/para/cmd/para@latest
```

## Quick start

```bash
mkdir ~/notes && cd ~/notes
para init
para inbox write --updated 2026-08-30 --body "Ship the PARA CLI"
para project create --id ship-para-cli \
  --status active \
  --due 2026-09-15 \
  --outcome "Public repo with installable binaries"
para area create --id personal-knowledge --status active --standard "Inbox reaches zero every week"
para resource create --id para-method --topic "PARA" --kind article
para review append --kind weekly --focus inbox
para project list
```

Project scope (default) writes vault files next to the working directory:

```text
INBOX.md
INDEX.md
Projects/
Areas/
Resources/
Archives/
.para/reviews.ndjson
```

Personal vault: `para --scope global ...` uses `~/.para/`.

## Commands

| Command | Role |
| --- | --- |
| `para init` | Create every declared path; existing files are left alone |
| `para inbox write` / `show` | Capture inbox |
| `para index write` / `show` | Vault overview |
| `para project create` / `list` / `get` / `update` / `delete` | Outcome-bound work |
| `para area create` / `list` / `get` / `update` / `delete` | Standing responsibilities |
| `para resource create` / `list` / `get` / `update` / `delete` | Topics to keep |
| `para archive create` / `list` / `get` / `update` / `delete` | Inactive items |
| `para review append` / `list` / `get` | Weekly / monthly sweep log |
| `para skills list` / `get` / `path` | Bundled Agent Skills (`para-notes`, `para-method`) |

`--scope project|global` is a persistent flag on every command.

## Agent Skills

The binary embeds two Skills. Print them or materialize a directory an agent can read:

```bash
para skills list
para skills get para-method
para skills path para-notes
```

`para-notes` is the ctxl-generated command guide. `para-method` is the filing rules for PARA.

## How this repo is built

1. Author PARA in [`context.schema.json`](context.schema.json).
2. `go generate ./...` runs `ctxl generate` and replaces [`cmd/para`](cmd/para).
3. Tag `v*` — [`.github/workflows/release.yml`](.github/workflows/release.yml) runs GoReleaser and publishes linux / darwin / windows / freebsd binaries (`amd64`, `arm64`).

Regenerate after a schema or Skill change:

```bash
go generate ./...
```

Do not edit `cmd/para` by hand. It is generated-owned.

## Desktop preview

[`desktop/`](desktop/) is a GPUI app that opens a vault: file tree, multi-tab read-only markdown, and a local PARA filing chat. First launch uses `~/para` and creates the PARA folders plus starter `INBOX.md` / `INDEX.md` if they are missing.

```bash
cd desktop
cargo run
cargo run -- example-vault
```

See [desktop/README.md](desktop/README.md). The CLI remains the writer.

## Local build

```bash
go build -o para ./cmd/para
./para --help
```

Requires Go 1.24.

## License

MIT
