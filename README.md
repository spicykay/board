# Board

A fast, local-first, Google Keep-style notes app. Plain-text markdown notes stored as
individual files, a polished **Tauri + SolidJS** desktop UI, and a scriptable `board`
CLI that agents can drive — all sharing a single Rust storage engine.

- **Local-first**: every note is a plain `.md` file you own, on your disk.
- **Fast**: SolidJS frontend (~18 KB JS) in a native Tauri window.
- **Two surfaces, one engine**: the GUI and the `board` CLI share `board-core`, so they
  can never disagree on format. Edit from either; the app live-reloads.
- **Pin, search, dark/light** — minimal by design.

## Project layout

A single Cargo workspace plus a SolidJS frontend:

```
crates/core/   # board-core: parse/write markdown + frontmatter, CRUD, search, watch
crates/cli/    # board-cli: the `board` binary — thin CLI over core, --json everywhere
src-tauri/     # Tauri backend: commands over core + filesystem watcher -> notes-changed
src/           # SolidJS + TS frontend (masonry grid, plain-text editor, dark/light)
bin/           # Hermit-managed toolchain (cargo, node, just) — committed, pins versions
Justfile       # local automation (see `just --list`)
```

## Storage

One `.md` file per note in the notes directory. Resolution order:
`$BOARD_DIR` → `notes_dir` in `<config-dir>/board/config.toml` → `~/Board` (default).

```markdown
---
id: 01J9X8...        # stable ULID — identity survives title/file renames
title: Grocery list
created: 2026-06-15T10:00:00Z
updated: 2026-06-15T10:05:00Z
pinned: true
---
buy milk
eggs
```

The filename is `slug-of-title-<shortid>.md` (human-readable); `id` is the source of truth.

## Prerequisites

The only thing you need is [**Hermit**](https://cashapp.github.io/hermit/) — it provides
the exact Rust, Node, and Just versions this project pins (no system installs required):

```sh
curl -fsSL https://github.com/cashapp/hermit/releases/download/stable/install.sh | bash
```

## Getting started

```sh
# Activate the project toolchain (adds ./bin to PATH). Optional but convenient:
. bin/activate-hermit       # or: source bin/activate-hermit

just setup                  # install JS dependencies
just dev                    # launch the desktop app (hot reload)
```

Not activated Hermit? Every recipe still works via `./bin/just <recipe>` — the Justfile
puts `./bin` on PATH for you.

## Automation (Just)

| Recipe              | What it does                                    |
| ------------------- | ----------------------------------------------- |
| `just dev`          | Run the desktop app in dev mode                 |
| `just build`        | Build the production app bundle                 |
| `just cli ...`      | Run the `board` CLI, e.g. `just cli list`       |
| `just cli-build`    | Build the release `board` binary                |
| `just cli-install`  | `cargo install` the `board` CLI onto your PATH  |
| `just test`         | Run the Rust test suite                         |
| `just check`        | Type-check Rust + the TypeScript frontend       |
| `just fmt`          | Format Rust + frontend                          |
| `just lint`         | Clippy (warnings as errors)                     |
| `just ci`           | `check` + `lint` + `test`                       |
| `just clean`        | Remove build artifacts                          |

## The `board` CLI

Build it with `just cli-build` (→ `target/release/board`) or run ad hoc via `just cli`.

A note can be referenced by its full `id`, its file stem, or its short-id suffix. Body
text comes from `--body`, piped stdin, or `$EDITOR` (in that order). Every read command
accepts `--json` for stable, machine-readable output — ideal for agents.

```sh
board new --title "Groceries" --body $'milk\neggs' --pin
echo "ship it" | board new --title "Work"
board list --json
board list --pinned
board show <ref> [--json]
board edit <ref>            # replace body (stdin/$EDITOR)
board append <ref>         # append to body
board rename <ref> "New title"
board pin <ref> / board unpin <ref>
board search "milk" [--json]
board rm <ref>
board path                 # print the active notes directory
board --dir /some/dir list # override the notes directory per-invocation
```

When the desktop app is open, any CLI change is picked up live via the filesystem
watcher (the backend emits `notes-changed` and the UI refetches).

## Roadmap (v2)

Labels/tags, note colors, a full-text search index, checklists, and device sync.
