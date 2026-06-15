# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Toolchain

All tooling (cargo, node, just) is pinned by **Hermit** under `./bin`, which every Just
recipe prepends to PATH. You do not need system installs. Either activate Hermit
(`. bin/activate-hermit`) so `just`/`cargo`/`npm` resolve, or call recipes via `./bin/just <recipe>`.

## Commands

Use the Justfile for everything; recipes set up PATH and dependencies for you.

- `just dev` — run the desktop app (hot-reloading UI + Rust). Runs `setup` first.
- `just build` — production desktop bundle.
- `just cli ...` — run the `board` CLI via cargo, e.g. `just cli list --json`.
- `just cli-build` — release `board` binary → `target/release/board`.
- `just test` — Rust test suite (`cargo test`).
- `just check` — `cargo check --workspace` + `npx tsc --noEmit`.
- `just lint` — `cargo clippy --workspace --all-targets -- -D warnings` (warnings are errors).
- `just fmt` — `cargo fmt` + prettier over `src/`.
- `just ci` — `check` + `lint` + `test`; run this before considering work done.

Run a single Rust test: `cargo test -p board-core <name>` (e.g. `cargo test -p board-core round_trips`).
Tests live inline in `#[cfg(test)]` modules in `crates/core`.

## Architecture

**One storage engine, three surfaces.** `crates/core` (`board-core`) is the single source
of truth for how notes are parsed from and written to disk. The desktop GUI (`src-tauri`)
and the `board` CLI (`crates/cli`) are both **thin layers** over it — they must never
reimplement storage, parsing, or note format. This is the core invariant: the GUI and CLI
can never disagree because they share `Store`.

```
crates/core/   board-core: Note model, markdown<->frontmatter (lib.rs), CRUD/search (store.rs),
               notes-dir resolution (config.rs), filesystem watcher (watch.rs)
crates/cli/    board-cli: the `board` binary — clap subcommands, --json output, stdin/$EDITOR input
src-tauri/     Tauri backend: #[tauri::command] fns wrapping Store + watcher -> `notes-changed` event
src/           SolidJS + TS frontend; src/api.ts is the typed bridge to the Tauri commands
```

**Note model.** Each note is one `.md` file: YAML frontmatter (`id`, `title`, `created`,
`updated`, `pinned`) plus a markdown body. The `id` is a **ULID and the source of truth for
identity** — it survives title/file renames. The filename is derived (`<slug-of-title>-<short-id>.md`)
and is purely cosmetic; renaming a title rewrites the file and deletes the old path
(`Store::update`), but the `id` stays put. `Note` (public, sent as JSON) and `Frontmatter`
(private, on-disk) are deliberately separate structs in `lib.rs`.

**A note "reference"** (used everywhere by CLI and commands) resolves against the full `id`,
the file stem, or the trailing short-id suffix — see `matches_reference` in `store.rs`.

**No index.** `Store::list()` reads and parses every `.md` file on each call, sorts
pinned-first then newest-first, and is the basis for `get`/`search`. Malformed/foreign files
are skipped, not fatal. Keep this in mind before adding features that assume an index.

**Live reload.** `board-core::watch` watches the notes dir (debounced) and the Tauri backend
emits `notes-changed`; the frontend subscribes via `onNotesChanged` (`src/api.ts`) and refetches.
This is how a CLI edit shows up live in the open app.

**Notes directory resolution** (`config.rs`): `$BOARD_DIR` → `notes_dir` in
`<config-dir>/board/config.toml` → `~/Board`. The CLI's `--dir` flag overrides per-invocation.

## Conventions

- **Adding a backend operation** requires touching three places in lockstep: a method on
  `Store`, a `#[tauri::command]` in `src-tauri/src/lib.rs` registered in the
  `invoke_handler!` list, and a typed wrapper in `src/api.ts`. Add a CLI subcommand in
  `crates/cli/src/main.rs` if it should be scriptable too.
- **Error handling:** `board-core` uses a typed `Error` enum (`thiserror`); the CLI uses
  `anyhow`; Tauri commands stringify errors for the frontend (`err()` helper).
- **CLI is agent-facing:** every read command supports `--json` for stable machine output.
  Body text resolves from `--body`, then piped stdin, then `$EDITOR` (in that order).
- The on-disk format normalizes to exactly one trailing newline on the body; preserve the
  `to_markdown`/`from_markdown` round-trip identity when changing serialization.
