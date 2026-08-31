# para

用 [ctxl](https://github.com/AkaraChen/ctxl) schema 生成的 [PARA](https://fortelabs.com/blog/para/) 笔记 CLI。

先记进 inbox，再把每条笔记归到 **Project / Area / Resource / Archive**。命令、存储布局、内置 Agent Skill 都以 [`context.schema.json`](context.schema.json) 为唯一来源。

## 安装

类 Unix（Linux、macOS、FreeBSD）：

```bash
curl -fsSL https://raw.githubusercontent.com/AkaraChen/para/main/scripts/install.sh | sh
```

Windows（PowerShell）：

```powershell
irm https://raw.githubusercontent.com/AkaraChen/para/main/scripts/install.ps1 | iex
```

脚本会下载对应平台的最新 GitHub Release，并校验 checksum。可用 `PARA_PREFIX` 改安装目录，或用 `PARA_VERSION=0.1.0` 固定版本。

已有 Go 1.24+ 也可以：

```bash
go install github.com/AkaraChen/para/cmd/para@latest
```

## 快速开始

```bash
mkdir ~/notes && cd ~/notes
para init
para inbox write --updated 2026-08-30 --body "把 PARA CLI 做完"
para project create --id ship-para-cli \
  --status active \
  --due 2026-09-15 \
  --outcome "公开仓库，能装到各平台 binary"
para area create --id personal-knowledge --status active --standard "每周 inbox 清零"
para resource create --id para-method --topic "PARA" --kind article
para review append --kind weekly --focus inbox
para project list
```

默认 project scope 把笔记写在当前目录：

```text
INBOX.md
INDEX.md
Projects/
Areas/
Resources/
Archives/
.para/reviews.ndjson
```

个人库：`para --scope global ...`，数据在 `~/.para/`。

## 桌面预览

[`desktop/`](desktop/) 是 GPUI 应用：左边文件树，中间多 tab 只读 Markdown，右边本地 PARA 归档对话。

```bash
cd desktop
cargo run -- example-vault
```

详见 [desktop/README.md](desktop/README.md)。写库仍用 CLI。

## 从 schema 生成

改 [`context.schema.json`](context.schema.json) 或 Skill 之后：

```bash
go generate ./...
```

不要手改 `cmd/para`，那是 ctxl 整目录替换的生成结果。推送 `v*` tag 后，GitHub Actions + GoReleaser 会发布各平台 binary。

## License

MIT
