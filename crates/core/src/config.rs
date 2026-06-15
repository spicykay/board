//! Resolution of the notes directory. Precedence:
//! 1. `$BOARD_DIR` environment variable
//! 2. `notes_dir` in the config file (`<config-dir>/board/config.toml`)
//! 3. Default: `~/Board`

use directories::{ProjectDirs, UserDirs};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct Config {
    notes_dir: Option<String>,
}

/// Resolve the active notes directory (does not create it).
pub fn default_root() -> PathBuf {
    if let Ok(dir) = std::env::var("BOARD_DIR") {
        if !dir.trim().is_empty() {
            return expand(&dir);
        }
    }
    if let Some(dir) = from_config_file() {
        return dir;
    }
    default_home_notes()
}

fn from_config_file() -> Option<PathBuf> {
    let proj = ProjectDirs::from("com", "kartikayalasomayajula", "board")?;
    let path = proj.config_dir().join("config.toml");
    let text = std::fs::read_to_string(path).ok()?;
    let cfg: Config = toml::from_str(&text).ok()?;
    cfg.notes_dir.map(|d| expand(&d))
}

fn default_home_notes() -> PathBuf {
    if let Some(dirs) = UserDirs::new() {
        return dirs.home_dir().join("Board");
    }
    PathBuf::from("Board")
}

/// Expand a leading `~` to the user's home directory.
fn expand(dir: &str) -> PathBuf {
    if let Some(rest) = dir.strip_prefix("~/") {
        if let Some(dirs) = UserDirs::new() {
            return dirs.home_dir().join(rest);
        }
    }
    PathBuf::from(dir)
}
