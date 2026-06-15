//! Tauri backend: thin command layer over `board-core`, plus a filesystem watcher
//! that pushes a `notes-changed` event to the UI when notes change on disk (e.g. the
//! `board` CLI writes a file while the app is open).

use board_core::{Note, Store};
use std::sync::Mutex;
use tauri::{Emitter, Manager};

/// App-wide state: the open note store.
struct AppState {
    store: Store,
}

/// Keeps the filesystem watcher alive for the lifetime of the app.
struct WatcherGuard(#[allow(dead_code)] Mutex<board_core::WatchHandle>);

/// Convert a core error into a string for the frontend.
fn err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

#[tauri::command]
fn list_notes(state: tauri::State<AppState>) -> Result<Vec<Note>, String> {
    state.store.list().map_err(err)
}

#[tauri::command]
fn get_note(reference: String, state: tauri::State<AppState>) -> Result<Note, String> {
    state.store.get(&reference).map_err(err)
}

#[tauri::command]
fn create_note(
    title: String,
    body: String,
    pinned: bool,
    state: tauri::State<AppState>,
) -> Result<Note, String> {
    state.store.create(&title, &body, pinned).map_err(err)
}

#[tauri::command]
fn update_note(
    reference: String,
    title: Option<String>,
    body: Option<String>,
    state: tauri::State<AppState>,
) -> Result<Note, String> {
    state
        .store
        .update(&reference, title.as_deref(), body.as_deref())
        .map_err(err)
}

#[tauri::command]
fn set_pinned(
    reference: String,
    pinned: bool,
    state: tauri::State<AppState>,
) -> Result<Note, String> {
    state.store.set_pinned(&reference, pinned).map_err(err)
}

#[tauri::command]
fn delete_note(reference: String, state: tauri::State<AppState>) -> Result<(), String> {
    state.store.delete(&reference).map_err(err)
}

#[tauri::command]
fn search_notes(query: String, state: tauri::State<AppState>) -> Result<Vec<Note>, String> {
    state.store.search(&query).map_err(err)
}

#[tauri::command]
fn notes_dir(state: tauri::State<AppState>) -> String {
    state.store.root().display().to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let store = Store::open_default().expect("failed to open notes store");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            store: store.clone(),
        })
        .setup(move |app| {
            // Live-reload: emit `notes-changed` when the notes directory changes
            // under us (CLI writes, external editors, sync clients, ...).
            let handle = app.handle().clone();
            let watcher = board_core::watch(store.root(), move || {
                let _ = handle.emit("notes-changed", ());
            })?;
            app.manage(WatcherGuard(Mutex::new(watcher)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_notes,
            get_note,
            create_note,
            update_note,
            set_pinned,
            delete_note,
            search_notes,
            notes_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
