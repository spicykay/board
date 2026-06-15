import {
  createEffect,
  createSignal,
  on,
  onCleanup,
  onMount,
  Show,
} from "solid-js";
import {
  createNote,
  deleteNote,
  listNotes,
  onNotesChanged,
  searchNotes,
  setPinned,
  type Note,
} from "./api";
import { Composer } from "./components/Composer";
import { Editor } from "./components/Editor";
import { NoteCard } from "./components/NoteCard";
import "./App.css";

type Theme = "light" | "dark";

function initialTheme(): Theme {
  const saved = localStorage.getItem("theme");
  if (saved === "light" || saved === "dark") return saved;
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function App() {
  const [notes, setNotes] = createSignal<Note[]>([]);
  const [query, setQuery] = createSignal("");
  const [selected, setSelected] = createSignal<Note | null>(null);
  const [theme, setTheme] = createSignal<Theme>(initialTheme());

  const refresh = async () => {
    const q = query().trim();
    setNotes(q ? await searchNotes(q) : await listNotes());
    // Keep the open editor's pin state in sync after external refreshes.
    const cur = selected();
    if (cur) {
      const fresh = notes().find((n) => n.id === cur.id);
      if (fresh) setSelected(fresh);
    }
  };

  createEffect(() => {
    document.documentElement.setAttribute("data-theme", theme());
    localStorage.setItem("theme", theme());
  });

  // Debounced re-query as the user types (initial load handled in onMount).
  createEffect(
    on(
      query,
      () => {
        const t = setTimeout(refresh, 150);
        onCleanup(() => clearTimeout(t));
      },
      { defer: true },
    ),
  );

  onMount(async () => {
    await refresh();
    const unlisten = await onNotesChanged(() => void refresh());
    onCleanup(() => unlisten());
  });

  const pinned = () => notes().filter((n) => n.pinned);
  const others = () => notes().filter((n) => !n.pinned);

  const create = async (title: string, body: string) => {
    await createNote(title, body, false);
    await refresh();
  };

  const togglePin = async (n: Note) => {
    const updated = await setPinned(n.id, !n.pinned);
    if (selected()?.id === n.id) setSelected(updated);
    await refresh();
  };

  const remove = async (n: Note) => {
    await deleteNote(n.id);
    if (selected()?.id === n.id) setSelected(null);
    await refresh();
  };

  const onSaved = (n: Note) => {
    setNotes((prev) => prev.map((p) => (p.id === n.id ? n : p)));
  };

  return (
    <div class="app">
      <header class="topbar">
        <div class="brand">
          <span class="brand-mark" />
          Board
        </div>
        <div class="search">
          <SearchIcon />
          <input
            placeholder="Search notes"
            value={query()}
            onInput={(e) => setQuery(e.currentTarget.value)}
          />
          <Show when={query()}>
            <button class="search-clear" onClick={() => setQuery("")} title="Clear">
              ×
            </button>
          </Show>
        </div>
        <button
          class="icon-btn theme-toggle"
          title="Toggle theme"
          onClick={() => setTheme(theme() === "dark" ? "light" : "dark")}
        >
          {theme() === "dark" ? <SunIcon /> : <MoonIcon />}
        </button>
      </header>

      <main class="content">
        <Composer onCreate={create} />

        <Show
          when={notes().length > 0}
          fallback={
            <p class="empty-state">
              {query() ? "No notes match your search." : "No notes yet — jot one down above."}
            </p>
          }
        >
          <Show when={pinned().length > 0}>
            <div class="section-label">Pinned</div>
            <div class="grid">
              {pinned().map((n) => (
                <NoteCard note={n} onOpen={setSelected} onTogglePin={togglePin} onDelete={remove} />
              ))}
            </div>
          </Show>

          <Show when={others().length > 0}>
            <Show when={pinned().length > 0}>
              <div class="section-label">Others</div>
            </Show>
            <div class="grid">
              {others().map((n) => (
                <NoteCard note={n} onOpen={setSelected} onTogglePin={togglePin} onDelete={remove} />
              ))}
            </div>
          </Show>
        </Show>
      </main>

      <Show when={selected()}>
        {(note) => (
          <Editor
            note={note()}
            onClose={() => setSelected(null)}
            onSaved={onSaved}
            onTogglePin={togglePin}
            onDelete={remove}
          />
        )}
      </Show>
    </div>
  );
}

function SearchIcon() {
  return (
    <svg viewBox="0 0 24 24" width="18" height="18" aria-hidden="true">
      <path
        d="M11 4a7 7 0 1 0 4.2 12.6L20 21M11 4a7 7 0 0 1 0 14"
        fill="none"
        stroke="currentColor"
        stroke-width="1.7"
        stroke-linecap="round"
      />
    </svg>
  );
}

function MoonIcon() {
  return (
    <svg viewBox="0 0 24 24" width="19" height="19" aria-hidden="true">
      <path
        d="M20 14.5A8 8 0 0 1 9.5 4 8 8 0 1 0 20 14.5z"
        fill="none"
        stroke="currentColor"
        stroke-width="1.7"
        stroke-linejoin="round"
      />
    </svg>
  );
}

function SunIcon() {
  return (
    <svg viewBox="0 0 24 24" width="19" height="19" aria-hidden="true">
      <circle cx="12" cy="12" r="4" fill="none" stroke="currentColor" stroke-width="1.7" />
      <path
        d="M12 2v2m0 16v2M2 12h2m16 0h2M5 5l1.5 1.5M17.5 17.5 19 19M19 5l-1.5 1.5M6.5 17.5 5 19"
        stroke="currentColor"
        stroke-width="1.7"
        stroke-linecap="round"
      />
    </svg>
  );
}

export default App;
