//! Filesystem watching so the GUI live-reloads when the CLI (or any editor) changes
//! a note on disk.

use crate::Result;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::time::{Duration, Instant};

/// An opaque handle that keeps a filesystem watch alive. Dropping it stops the
/// watch. Returned by [`watch`] so callers don't need a direct `notify` dependency.
pub struct WatchHandle(#[allow(dead_code)] RecommendedWatcher);

/// Watch `root` for note changes, invoking `on_change` (coalesced) whenever a
/// relevant `.md` event fires.
///
/// The returned [`WatchHandle`] must be kept alive for watching to continue;
/// dropping it stops the watch.
pub fn watch<F>(root: impl AsRef<Path>, on_change: F) -> Result<WatchHandle>
where
    F: Fn() + Send + 'static,
{
    // Coalesce bursts (a single save can emit several events) into one callback.
    let mut last = Instant::now() - Duration::from_secs(1);
    let debounce = Duration::from_millis(150);

    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        let Ok(event) = res else { return };
        let touches_md = event
            .paths
            .iter()
            .any(|p| p.extension().and_then(|e| e.to_str()) == Some("md"));
        if !touches_md {
            return;
        }
        if last.elapsed() >= debounce {
            last = Instant::now();
            on_change();
        }
    })
    .map_err(to_io)?;

    watcher
        .watch(root.as_ref(), RecursiveMode::NonRecursive)
        .map_err(to_io)?;
    Ok(WatchHandle(watcher))
}

fn to_io(e: notify::Error) -> crate::Error {
    crate::Error::Io(std::io::Error::other(e.to_string()))
}
