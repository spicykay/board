import { Show } from "solid-js";
import type { Note } from "../api";

export function NoteCard(props: {
  note: Note;
  onOpen: (n: Note) => void;
  onTogglePin: (n: Note) => void;
  onDelete: (n: Note) => void;
}) {
  const empty = () => !props.note.title && !props.note.body;
  return (
    <article class="card" onClick={() => props.onOpen(props.note)}>
      <button
        class="pin"
        classList={{ active: props.note.pinned }}
        title={props.note.pinned ? "Unpin" : "Pin"}
        onClick={(e) => {
          e.stopPropagation();
          props.onTogglePin(props.note);
        }}
      >
        <PinIcon filled={props.note.pinned} />
      </button>

      <Show when={props.note.title}>
        <h3 class="card-title">{props.note.title}</h3>
      </Show>
      <Show when={props.note.body}>
        <p class="card-body">{props.note.body}</p>
      </Show>
      <Show when={empty()}>
        <p class="card-empty">Empty note</p>
      </Show>

      <div class="card-actions">
        <button
          class="icon-btn"
          title="Delete"
          onClick={(e) => {
            e.stopPropagation();
            props.onDelete(props.note);
          }}
        >
          <TrashIcon />
        </button>
      </div>
    </article>
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

function TrashIcon() {
  return (
    <svg viewBox="0 0 24 24" width="17" height="17" aria-hidden="true">
      <path
        d="M5 7h14M10 7V5h4v2m-7 0 1 12h8l1-12"
        fill="none"
        stroke="currentColor"
        stroke-width="1.6"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
  );
}
