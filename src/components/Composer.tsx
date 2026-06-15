import { createSignal, Show } from "solid-js";

/** Google Keep-style "Take a note…" composer that expands on focus. */
export function Composer(props: { onCreate: (title: string, body: string) => void }) {
  const [open, setOpen] = createSignal(false);
  const [title, setTitle] = createSignal("");
  const [body, setBody] = createSignal("");
  let bodyRef: HTMLTextAreaElement | undefined;

  const reset = () => {
    setTitle("");
    setBody("");
    setOpen(false);
  };

  const submit = () => {
    if (title().trim() || body().trim()) {
      props.onCreate(title().trim(), body());
    }
    reset();
  };

  return (
    <div class="composer" classList={{ open: open() }}>
      <Show
        when={open()}
        fallback={
          <button
            class="composer-collapsed"
            onClick={() => {
              setOpen(true);
              queueMicrotask(() => bodyRef?.focus());
            }}
          >
            Take a note…
          </button>
        }
      >
        <input
          class="composer-title"
          placeholder="Title"
          value={title()}
          onInput={(e) => setTitle(e.currentTarget.value)}
        />
        <textarea
          ref={bodyRef}
          class="composer-body"
          placeholder="Take a note…"
          rows={3}
          value={body()}
          onInput={(e) => setBody(e.currentTarget.value)}
          onKeyDown={(e) => {
            if (e.key === "Escape") reset();
          }}
        />
        <div class="composer-actions">
          <button class="btn-text" onClick={submit}>
            Done
          </button>
        </div>
      </Show>
    </div>
  );
}
