package para_test

import (
	"encoding/json"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"
)

func TestParaVaultSmoke(t *testing.T) {
	bin := filepath.Join(t.TempDir(), "para")
	build := exec.Command("go", "build", "-o", bin, "./cmd/para")
	build.Dir = repoRoot(t)
	if out, err := build.CombinedOutput(); err != nil {
		t.Fatalf("go build: %v\n%s", err, out)
	}

	vault := t.TempDir()
	run := func(args ...string) string {
		t.Helper()
		cmd := exec.Command(bin, args...)
		cmd.Dir = vault
		out, err := cmd.CombinedOutput()
		if err != nil {
			t.Fatalf("para %s: %v\n%s", strings.Join(args, " "), err, out)
		}
		return string(out)
	}

	help := run("--help")
	for _, want := range []string{"inbox", "project", "area", "resource", "archive", "review", "skills"} {
		if !strings.Contains(help, want) {
			t.Fatalf("help missing %q:\n%s", want, help)
		}
	}
	if strings.Contains(help, "--schema") {
		t.Fatalf("runtime help exposes --schema:\n%s", help)
	}

	initOut := run("init")
	if !strings.Contains(initOut, `"action": "created"`) {
		t.Fatalf("init = %s", initOut)
	}
	for _, path := range []string{"INBOX.md", "INDEX.md", "Projects", "Areas", "Resources", "Archives", filepath.Join(".para", "reviews.ndjson")} {
		if _, err := os.Stat(filepath.Join(vault, path)); err != nil {
			t.Fatalf("missing %s: %v", path, err)
		}
	}

	run("inbox", "write", "--updated", "2026-08-30", "--body", "Ship the PARA CLI")
	inbox := run("inbox", "show")
	if !strings.Contains(inbox, "Ship the PARA CLI") {
		t.Fatalf("inbox show = %s", inbox)
	}

	run("project", "create", "--id", "ship-para-cli", "--status", "active", "--outcome", "Public repo with binaries", "--due", "2026-09-15")
	run("area", "create", "--id", "personal-knowledge", "--status", "active", "--standard", "Inbox reaches zero every week")
	run("resource", "create", "--id", "para-method", "--topic", "PARA", "--kind", "article")
	run("archive", "create", "--id", "old-side-project", "--origin", "project", "--archived", "2026-01-01")
	run("review", "append", "--kind", "weekly", "--focus", "inbox")

	var projects []string
	if err := json.Unmarshal([]byte(run("project", "list")), &projects); err != nil {
		t.Fatal(err)
	}
	if len(projects) != 1 || projects[0] != "ship-para-cli" {
		t.Fatalf("project list = %v", projects)
	}

	skills := run("skills", "list")
	if !strings.Contains(skills, "para-notes") || !strings.Contains(skills, "para-method") {
		t.Fatalf("skills list = %s", skills)
	}
	method := run("skills", "get", "para-method")
	if !strings.Contains(method, "PARA is a filing system") {
		t.Fatalf("para-method skill missing body:\n%s", method)
	}
}

func repoRoot(t *testing.T) {
	t.Helper()
	wd, err := os.Getwd()
	if err != nil {
		t.Fatal(err)
	}
	return wd
}
