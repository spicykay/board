import { createSignal, onCleanup, onMount } from "solid-js";
import { updateNote, type Note } from "../api";

/** Expanded note editor (modal). Plain-text body with debounced autosave. */
export function Editor(props: {
  note: Note;
  onClose: () => void;
  onSaved: (n: Note) => void;
  onTogglePin: (n: Note) => void;
  onDelete: (n: Note) => void;
}) {
  const [title, setTitle] = createSignal(props.note.title);
  const [body, setBody] = createSignal(props.note.body);
  let timer: ReturnType<typeof setTimeout> | undefined;
  let dirty = false;

  const save = async () => {
    if (!dirty) return;
    dirty = false;
    const n = await updateNote(props.note.id, title(), body());
    props.onSaved(n);
  };

  const scheduleSave = () => {
    dirty = true;
    clearTimeout(timer);
    timer = setTimeout(save, 400);
  };

  const close = async () => {
    clearTimeout(timer);
    await save();
    props.onClose();
  };

  const onKey = (e: KeyboardEvent) => {
    if (e.key === "Escape") void close();
  };
  onMount(() => window.addEventListener("keydown", onKey));
  onCleanup(() => {
    window.removeEventListener("keydown", onKey);
    clearTimeout(timer);
  });

  return (
    <div class="modal-backdrop" onClick={() => void close()}>
      <div class="modal" onClick={(e) => e.stopPropagation()}>
        <div class="modal-head">
          <input
            class="modal-title"
            placeholder="Title"
            value={title()}
            onInput={(e) => {
              setTitle(e.currentTarget.value);
              scheduleSave();
            }}
          />
          <button
            class="pin"
            classList={{ active: props.note.pinned }}
            title={props.note.pinned ? "Unpin" : "Pin"}
            onClick={() => props.onTogglePin(props.note)}
          >
            <PinIcon filled={props.note.pinned} />
          </button>
        </div>
        <textarea
          class="modal-body"
          placeholder="Note"
          value={body()}
          onInput={(e) => {
            setBody(e.currentTarget.value);
            scheduleSave();
          }}
        />
        <div class="modal-foot">
          <button class="btn-text danger" onClick={() => props.onDelete(props.note)}>
            Delete
          </button>
          <button class="btn-text" onClick={() => void close()}>
            Close
          </button>
        </div>
      </div>
    </div>
  );
}

function PinIcon(props: { filled: boolean }) {
  return (
    <svg viewBox="0 0 24 24" width="18" height="18" aria-hidden="true">
      <path
        d="M14 4v6l3 3v2h-5v5l-1 1-1-1v-5H4v-2l3-3V4H6V2h12v2h-2z"
        fill={props.filled ? "currentColor" : "none"}
        stroke="currentColor"
        stroke-width="1.6"
        stroke-linejoin="round"
      />
    </svg>
  );
}
