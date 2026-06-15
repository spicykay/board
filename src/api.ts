// Typed wrappers over the Tauri commands exposed by `src-tauri/src/lib.rs`,
// plus a subscription to the backend's `notes-changed` live-reload event.
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface Note {
  id: string;
  title: string;
  created: string;
  updated: string;
  pinned: boolean;
  body: string;
  path: string;
}

export const listNotes = () => invoke<Note[]>("list_notes");

export const searchNotes = (query: string) =>
  invoke<Note[]>("search_notes", { query });

export const createNote = (title: string, body: string, pinned: boolean) =>
  invoke<Note>("create_note", { title, body, pinned });

export const updateNote = (
  reference: string,
  title: string | null,
  body: string | null,
) => invoke<Note>("update_note", { reference, title, body });

export const setPinned = (reference: string, pinned: boolean) =>
  invoke<Note>("set_pinned", { reference, pinned });

export const deleteNote = (reference: string) =>
  invoke<void>("delete_note", { reference });

export const notesDir = () => invoke<string>("notes_dir");

/** Subscribe to filesystem-driven changes (e.g. the `notes` CLI). */
export const onNotesChanged = (cb: () => void): Promise<UnlistenFn> =>
  listen("notes-changed", () => cb());
