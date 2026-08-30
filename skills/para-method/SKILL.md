---
name: para-method
description: File notes with Tiago Forte's PARA method — Projects, Areas, Resources, Archives — using the para CLI.
---

# PARA method

PARA is a filing system, not a writing system. Capture first, then put each note in exactly one of four buckets.

| Bucket | Question | Lives until |
| --- | --- | --- |
| **Project** | Does this have a finish line and a concrete outcome? | The outcome is shipped, then archive it |
| **Area** | Is this a standing responsibility with a standard to keep? | The role ends |
| **Resource** | Is this a topic you want to keep around? | It stops being useful, then archive it |
| **Archive** | Is this inactive but worth keeping? | Forever, for search |

Never invent a fifth top-level bucket. If it is not a project, area, or resource, it is either still in the inbox or it belongs in the archive.

## Capture, then file

1. Dump raw notes into the inbox. Do not classify while capturing.
2. On a review pass, give each inbox item one home:
   - a deadline + outcome → `para project create`
   - an ongoing role → `para area create`
   - a topic to keep → `para resource create`
   - done or inactive → `para archive create`
3. Clear the inbox body after everything is filed.
4. Append a review row so the sweep is visible later.

Use `para`; do not create vault folders or markdown files by hand.

## Decision rules

- A project without a due date or outcome is an area in disguise. Fix the fields, do not keep it as a project.
- An area with a finish line is a project. Move it.
- Resources are references. They do not have owners or deadlines.
- When a project is `done`, copy the useful body into an archive item with `--origin project` and delete or leave the project as `done` only until the next review.
- Prefer kebab-case ids: `ship-para-cli`, `health`, `golang-generics`.

## Review cadence

- **Weekly:** inbox to zero, project statuses, what moved to archive.
- **Monthly:** area standards still true? resources still used?
- Log every sweep with `para review append --kind weekly|monthly|ad-hoc`.

## Scope

- Project vault (default): files sit next to the working directory (`INBOX.md`, `Projects/`, …). Review rows go in `.para/reviews.ndjson`.
- Personal vault: `para --scope global ...` stores the same layout under `~/.para/`.
