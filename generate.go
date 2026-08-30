// Package para holds the go:generate entrypoint for the PARA CLI.
package para

import _ "github.com/AkaraChen/ctxl/cli"

//go:generate go run github.com/AkaraChen/ctxl/cmd/ctxl@v0.0.0-20260827070639-933b52740aab generate context.schema.json
