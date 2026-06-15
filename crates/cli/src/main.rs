//! `board` — a small, scriptable CLI over the shared `board-core` store.
//!
//! Every read command supports `--json` so agents get a stable, machine-readable
//! contract. Text comes from `--body`, piped stdin, or `$EDITOR` (in that order).

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use board_core::{Note, Store};
use std::io::{IsTerminal, Read, Write};

#[derive(Parser)]
#[command(
    name = "board",
    about = "Local-first markdown notes — shared with the Board desktop app.",
    version
)]
struct Cli {
    /// Override the notes directory (defaults to $BOARD_DIR, config, or ~/Board).
    #[arg(long, global = true)]
    dir: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a new note.
    New {
        #[arg(long)]
        title: Option<String>,
        /// Pin the new note.
        #[arg(long)]
        pin: bool,
        #[command(flatten)]
        input: BodyInput,
        #[arg(long)]
        json: bool,
    },
    /// List all notes (pinned first, newest first).
    List {
        /// Only show pinned notes.
        #[arg(long)]
        pinned: bool,
        #[arg(long)]
        json: bool,
    },
    /// Show a single note by id, file stem, or short-id.
    Show {
        reference: String,
        #[arg(long)]
        json: bool,
    },
    /// Replace a note's body.
    Edit {
        reference: String,
        #[command(flatten)]
        input: BodyInput,
        #[arg(long)]
        json: bool,
    },
    /// Append text to a note's body.
    Append {
        reference: String,
        #[command(flatten)]
        input: BodyInput,
        #[arg(long)]
        json: bool,
    },
    /// Rename a note (change its title).
    Rename {
        reference: String,
        title: String,
        #[arg(long)]
        json: bool,
    },
    /// Pin a note.
    Pin { reference: String },
    /// Unpin a note.
    Unpin { reference: String },
    /// Delete a note.
    Rm { reference: String },
    /// Full-text search over titles and bodies.
    Search {
        query: String,
        #[arg(long)]
        json: bool,
    },
    /// Print the active notes directory.
    Path,
}

#[derive(Args)]
struct BodyInput {
    /// Provide the note body inline (skips stdin/$EDITOR).
    #[arg(long)]
    body: Option<String>,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("notes: {e:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let store = match &cli.dir {
        Some(d) => Store::open(d),
        None => Store::open_default(),
    }
    .context("opening notes store")?;

    match cli.command {
        Command::New {
            title,
            pin,
            input,
            json,
        } => {
            let title = title.unwrap_or_default();
            let body = input.resolve("")?;
            let note = store.create(&title, &body, pin)?;
            emit(&note, json, "created");
        }
        Command::List { pinned, json } => {
            let mut notes = store.list()?;
            if pinned {
                notes.retain(|n| n.pinned);
            }
            emit_list(&notes, json);
        }
        Command::Show { reference, json } => {
            let note = store.get(&reference)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&note)?);
            } else {
                print!("{}", note.body);
                if !note.body.ends_with('\n') {
                    println!();
                }
            }
        }
        Command::Edit {
            reference,
            input,
            json,
        } => {
            let current = store.get(&reference)?;
            let body = input.resolve(&current.body)?;
            let note = store.update(&reference, None, Some(&body))?;
            emit(&note, json, "updated");
        }
        Command::Append {
            reference,
            input,
            json,
        } => {
            let current = store.get(&reference)?;
            let addition = input.resolve("")?;
            let mut body = current.body.clone();
            if !body.is_empty() && !body.ends_with('\n') {
                body.push('\n');
            }
            body.push_str(&addition);
            let note = store.update(&reference, None, Some(&body))?;
            emit(&note, json, "updated");
        }
        Command::Rename {
            reference,
            title,
            json,
        } => {
            let note = store.update(&reference, Some(&title), None)?;
            emit(&note, json, "renamed");
        }
        Command::Pin { reference } => {
            let note = store.set_pinned(&reference, true)?;
            emit(&note, false, "pinned");
        }
        Command::Unpin { reference } => {
            let note = store.set_pinned(&reference, false)?;
            emit(&note, false, "unpinned");
        }
        Command::Rm { reference } => {
            store.delete(&reference)?;
            println!("deleted {reference}");
        }
        Command::Search { query, json } => {
            let notes = store.search(&query)?;
            emit_list(&notes, json);
        }
        Command::Path => {
            println!("{}", store.root().display());
        }
    }
    Ok(())
}

impl BodyInput {
    /// Resolve note text from `--body`, then piped stdin, then `$EDITOR`
    /// (pre-filled with `seed`), falling back to `seed` if nothing is provided.
    fn resolve(&self, seed: &str) -> Result<String> {
        if let Some(body) = &self.body {
            return Ok(body.clone());
        }
        let stdin = std::io::stdin();
        if !stdin.is_terminal() {
            let mut buf = String::new();
            stdin.lock().read_to_string(&mut buf)?;
            return Ok(buf.trim_end_matches('\n').to_string());
        }
        edit_in_editor(seed)
    }
}

/// Open `$EDITOR` on a temp file pre-filled with `seed` and return the result.
fn edit_in_editor(seed: &str) -> Result<String> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    let mut path = std::env::temp_dir();
    path.push(format!("notes-{}.md", std::process::id()));
    std::fs::write(&path, seed)?;

    let status = std::process::Command::new(&editor)
        .arg(&path)
        .status()
        .with_context(|| format!("launching editor `{editor}`"))?;
    if !status.success() {
        anyhow::bail!("editor exited with a non-zero status");
    }
    let text = std::fs::read_to_string(&path)?;
    let _ = std::fs::remove_file(&path);
    Ok(text.trim_end_matches('\n').to_string())
}

/// Print a single note (JSON or a one-line confirmation).
fn emit(note: &Note, json: bool, verb: &str) {
    if json {
        let out = serde_json::to_string(note).expect("serialize note");
        println!("{out}");
    } else {
        println!("{verb} {}  {}", short(&note.id), note.title);
    }
}

/// Print a list of notes (JSON array or aligned text).
fn emit_list(notes: &[Note], json: bool) {
    if json {
        let out = serde_json::to_string_pretty(notes).expect("serialize notes");
        println!("{out}");
        return;
    }
    let mut out = std::io::stdout().lock();
    for n in notes {
        let pin = if n.pinned { "📌" } else { "  " };
        let title = if n.title.is_empty() { "(untitled)" } else { &n.title };
        let _ = writeln!(out, "{pin} {}  {title}", short(&n.id));
    }
}

fn short(id: &str) -> &str {
    let n = id.len().min(6);
    &id[id.len() - n..]
}
