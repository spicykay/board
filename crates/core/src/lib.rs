//! `board-core` — the local-first markdown storage engine shared by the desktop
//! GUI (`src-tauri`) and the `board` CLI. It is the single source of truth for how
//! notes are parsed from and written to disk, so the two surfaces can never drift.
//!
//! Each note is one `.md` file with a YAML frontmatter block:
//!
//! ```text
//! ---
//! id: 01J9X8...
//! title: Grocery list
//! created: 2026-06-15T10:00:00Z
//! updated: 2026-06-15T10:05:00Z
//! pinned: true
//! ---
//! buy milk
//! eggs
//! ```

mod config;
mod store;
mod watch;

pub use config::default_root;
pub use store::Store;
pub use watch::{watch, WatchHandle};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Errors surfaced by the storage engine.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("note not found: {0}")]
    NotFound(String),
    #[error("malformed note (missing or invalid frontmatter): {0}")]
    Malformed(PathBuf),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("yaml error: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

/// A single note. This is the shape sent to the frontend (as JSON) and printed by
/// the CLI's `--json` mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub pinned: bool,
    pub body: String,
    /// Absolute path of the backing `.md` file.
    pub path: PathBuf,
}

/// The on-disk frontmatter portion of a note (everything except the body).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Frontmatter {
    id: String,
    title: String,
    created: DateTime<Utc>,
    updated: DateTime<Utc>,
    #[serde(default)]
    pinned: bool,
}

impl Note {
    /// Render the note to its on-disk string form (frontmatter + body).
    fn to_markdown(&self) -> Result<String> {
        let fm = Frontmatter {
            id: self.id.clone(),
            title: self.title.clone(),
            created: self.created,
            updated: self.updated,
            pinned: self.pinned,
        };
        // `serde_yaml::to_string` already terminates with a newline.
        let yaml = serde_yaml::to_string(&fm)?;
        let mut body = self.body.clone();
        // Keep files tidy: exactly one trailing newline.
        while body.ends_with('\n') {
            body.pop();
        }
        Ok(format!("---\n{yaml}---\n{body}\n"))
    }

    /// Parse a note from its on-disk string form.
    fn from_markdown(content: &str, path: PathBuf) -> Result<Note> {
        let (fm_str, body) =
            split_frontmatter(content).ok_or_else(|| Error::Malformed(path.clone()))?;
        let fm: Frontmatter = serde_yaml::from_str(fm_str)?;
        // `to_markdown` writes exactly one trailing newline; strip it so the
        // round-trip is an identity on the logical body content.
        let body = body.strip_suffix('\n').unwrap_or(body);
        let body = body.strip_suffix('\r').unwrap_or(body);
        Ok(Note {
            id: fm.id,
            title: fm.title,
            created: fm.created,
            updated: fm.updated,
            pinned: fm.pinned,
            body: body.to_string(),
            path,
        })
    }
}

/// Split a document into its `(frontmatter, body)` halves. Returns `None` if the
/// document does not open with a `---` fenced YAML block.
fn split_frontmatter(content: &str) -> Option<(&str, &str)> {
    // Tolerate a leading BOM / CR but require the opening fence on the first line.
    let content = content.strip_prefix('\u{feff}').unwrap_or(content);
    let after_open = content.strip_prefix("---\n").or_else(|| content.strip_prefix("---\r\n"))?;

    // The closing fence is a line that is exactly `---`.
    for (idx, _) in after_open.match_indices("---") {
        let at_line_start = idx == 0 || after_open[..idx].ends_with('\n');
        let rest = &after_open[idx + 3..];
        let closes_line = rest.is_empty() || rest.starts_with('\n') || rest.starts_with("\r\n");
        if at_line_start && closes_line {
            let fm = &after_open[..idx];
            let body = rest
                .strip_prefix('\n')
                .or_else(|| rest.strip_prefix("\r\n"))
                .unwrap_or(rest);
            return Some((fm, body));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(id: &str) -> Note {
        let ts = "2026-06-15T10:00:00Z".parse().unwrap();
        Note {
            id: id.into(),
            title: "Grocery list".into(),
            created: ts,
            updated: ts,
            pinned: true,
            body: "buy milk\neggs".into(),
            path: PathBuf::from("/tmp/x.md"),
        }
    }

    #[test]
    fn round_trips_through_markdown() {
        let note = sample("01ABC");
        let md = note.to_markdown().unwrap();
        assert!(md.starts_with("---\n"));
        let parsed = Note::from_markdown(&md, PathBuf::from("/tmp/x.md")).unwrap();
        assert_eq!(parsed.id, "01ABC");
        assert_eq!(parsed.title, "Grocery list");
        assert_eq!(parsed.body, "buy milk\neggs");
        assert!(parsed.pinned);
    }

    #[test]
    fn parses_empty_body() {
        let md = "---\nid: a\ntitle: t\ncreated: 2026-06-15T10:00:00Z\nupdated: 2026-06-15T10:00:00Z\npinned: false\n---\n";
        let parsed = Note::from_markdown(md, PathBuf::from("/tmp/x.md")).unwrap();
        assert_eq!(parsed.body, "");
        assert!(!parsed.pinned);
    }

    #[test]
    fn rejects_missing_frontmatter() {
        let err = Note::from_markdown("just text", PathBuf::from("/tmp/x.md"));
        assert!(err.is_err());
    }
}
