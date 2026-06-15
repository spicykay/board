//! The note store: CRUD over a directory of `.md` files.

use crate::{Error, Note, Result};
use chrono::Utc;
use std::path::{Path, PathBuf};

/// A handle to a directory of notes.
#[derive(Debug, Clone)]
pub struct Store {
    root: PathBuf,
}

impl Store {
    /// Open (creating if needed) a store rooted at `root`.
    pub fn open(root: impl Into<PathBuf>) -> Result<Store> {
        let root = root.into();
        std::fs::create_dir_all(&root)?;
        Ok(Store { root })
    }

    /// Open the store at the configured default location (`$BOARD_DIR` / config / `~/Board`).
    pub fn open_default() -> Result<Store> {
        Store::open(crate::default_root())
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// All notes, sorted pinned-first then most-recently-updated.
    pub fn list(&self) -> Result<Vec<Note>> {
        let mut notes = Vec::new();
        for entry in std::fs::read_dir(&self.root)? {
            let path = entry?.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            let content = std::fs::read_to_string(&path)?;
            // Skip foreign/malformed files rather than failing the whole listing.
            if let Ok(note) = Note::from_markdown(&content, path) {
                notes.push(note);
            }
        }
        notes.sort_by(|a, b| {
            b.pinned
                .cmp(&a.pinned)
                .then(b.updated.cmp(&a.updated))
        });
        Ok(notes)
    }

    /// Resolve a note by its id, its file stem, or its trailing short-id suffix.
    pub fn get(&self, reference: &str) -> Result<Note> {
        let notes = self.list()?;
        notes
            .into_iter()
            .find(|n| matches_reference(n, reference))
            .ok_or_else(|| Error::NotFound(reference.to_string()))
    }

    /// Create a new note and write it to disk.
    pub fn create(&self, title: &str, body: &str, pinned: bool) -> Result<Note> {
        let now = Utc::now();
        let id = ulid::Ulid::new().to_string();
        let note = Note {
            path: self.path_for(&id, title),
            id,
            title: title.to_string(),
            created: now,
            updated: now,
            pinned,
            body: body.to_string(),
        };
        self.write(&note)?;
        Ok(note)
    }

    /// Update a note's title and/or body. Passing `None` leaves a field unchanged.
    pub fn update(&self, reference: &str, title: Option<&str>, body: Option<&str>) -> Result<Note> {
        let mut note = self.get(reference)?;
        let old_path = note.path.clone();
        if let Some(t) = title {
            note.title = t.to_string();
            // Keep the filename in sync with the title (identity stays in `id`).
            note.path = self.path_for(&note.id, t);
        }
        if let Some(b) = body {
            note.body = b.to_string();
        }
        note.updated = Utc::now();
        self.write(&note)?;
        if note.path != old_path && old_path.exists() {
            std::fs::remove_file(&old_path)?;
        }
        Ok(note)
    }

    /// Pin or unpin a note.
    pub fn set_pinned(&self, reference: &str, pinned: bool) -> Result<Note> {
        let mut note = self.get(reference)?;
        note.pinned = pinned;
        note.updated = Utc::now();
        self.write(&note)?;
        Ok(note)
    }

    /// Delete a note.
    pub fn delete(&self, reference: &str) -> Result<()> {
        let note = self.get(reference)?;
        std::fs::remove_file(&note.path)?;
        Ok(())
    }

    /// Case-insensitive substring search over title and body.
    pub fn search(&self, query: &str) -> Result<Vec<Note>> {
        let needle = query.to_lowercase();
        Ok(self
            .list()?
            .into_iter()
            .filter(|n| {
                n.title.to_lowercase().contains(&needle)
                    || n.body.to_lowercase().contains(&needle)
            })
            .collect())
    }

    fn write(&self, note: &Note) -> Result<()> {
        let md = note.to_markdown()?;
        std::fs::write(&note.path, md)?;
        Ok(())
    }

    /// Build a human-readable, collision-free filename: `<slug>-<short-id>.md`.
    fn path_for(&self, id: &str, title: &str) -> PathBuf {
        let mut slug = slug::slugify(title);
        if slug.is_empty() {
            slug = "untitled".to_string();
        }
        let short = short_id(id);
        self.root.join(format!("{slug}-{short}.md"))
    }
}

/// The trailing portion of the id used as a stable, unique filename suffix.
fn short_id(id: &str) -> String {
    let n = id.len().min(6);
    id[id.len() - n..].to_lowercase()
}

fn matches_reference(note: &Note, reference: &str) -> bool {
    if note.id.eq_ignore_ascii_case(reference) {
        return true;
    }
    if let Some(stem) = note.path.file_stem().and_then(|s| s.to_str()) {
        if stem.eq_ignore_ascii_case(reference) {
            return true;
        }
    }
    // Allow referring to a note by just its short-id suffix.
    short_id(&note.id).eq_ignore_ascii_case(reference)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_store() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path()).unwrap();
        (dir, store)
    }

    #[test]
    fn create_list_get_roundtrip() {
        let (_d, store) = temp_store();
        let n = store.create("Hello", "world", false).unwrap();
        assert_eq!(store.list().unwrap().len(), 1);
        let fetched = store.get(&n.id).unwrap();
        assert_eq!(fetched.body, "world");
        // resolvable by short-id suffix too
        assert!(store.get(&super::short_id(&n.id)).is_ok());
    }

    #[test]
    fn pinned_sorts_first() {
        let (_d, store) = temp_store();
        let _a = store.create("a", "", false).unwrap();
        let b = store.create("b", "", false).unwrap();
        store.set_pinned(&b.id, true).unwrap();
        let list = store.list().unwrap();
        assert_eq!(list[0].id, b.id);
        assert!(list[0].pinned);
    }

    #[test]
    fn update_title_renames_file_keeps_id() {
        let (_d, store) = temp_store();
        let n = store.create("First", "body", false).unwrap();
        let old_path = n.path.clone();
        let updated = store.update(&n.id, Some("Second"), None).unwrap();
        assert_eq!(updated.id, n.id);
        assert_ne!(updated.path, old_path);
        assert!(!old_path.exists());
        assert!(updated.path.exists());
        assert_eq!(store.list().unwrap().len(), 1);
    }

    #[test]
    fn search_matches_title_and_body() {
        let (_d, store) = temp_store();
        store.create("Groceries", "milk and eggs", false).unwrap();
        store.create("Work", "ship the thing", false).unwrap();
        assert_eq!(store.search("MILK").unwrap().len(), 1);
        assert_eq!(store.search("the").unwrap().len(), 1);
        assert_eq!(store.search("zzz").unwrap().len(), 0);
    }

    #[test]
    fn delete_removes_note() {
        let (_d, store) = temp_store();
        let n = store.create("x", "y", false).unwrap();
        store.delete(&n.id).unwrap();
        assert_eq!(store.list().unwrap().len(), 0);
    }
}
