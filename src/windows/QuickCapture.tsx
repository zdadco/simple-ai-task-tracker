import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { createTask, getDictationHint, hideWindow } from "../lib/tauri";

export default function QuickCapture() {
  const [title, setTitle] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const inputRef = useRef<HTMLTextAreaElement>(null);

  const focusInput = useCallback(() => {
    inputRef.current?.focus();
    inputRef.current?.select();
  }, []);

  useEffect(() => {
    focusInput();
  }, [focusInput]);

  useEffect(() => {
    const unlisten = listen("quick-capture-focus", () => {
      focusInput();
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [focusInput]);

  useEffect(() => {
    const handleKey = async (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        await hideWindow("quick-capture");
      }
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, []);

  async function handleSubmit() {
    const trimmed = title.trim();
    if (!trimmed || submitting) return;

    setSubmitting(true);
    try {
      await createTask(trimmed);
      setTitle("");
      await hideWindow("quick-capture");
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <div
      className="flex h-full flex-col bg-white p-3"
      onMouseDown={(e) => {
        // Allow dragging frameless window
        if (e.buttons === 1 && (e.target as HTMLElement).dataset.drag !== "false") {
          getCurrentWindow().startDragging();
        }
      }}
    >
      <textarea
        ref={inputRef}
        value={title}
        onChange={(e) => setTitle(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter" && !e.shiftKey) {
            e.preventDefault();
            handleSubmit();
          }
        }}
        placeholder="Новая задача... (Enter — добавить, Esc — закрыть)"
        className="flex-1 resize-none rounded border border-gray-200 p-2 text-sm outline-none focus:border-indigo-400"
        data-drag="false"
        rows={3}
      />

      <div className="mt-2 flex items-center justify-between">
        <p className="text-xs text-gray-400">{getDictationHint()}</p>
        <button
          type="button"
          onClick={handleSubmit}
          disabled={!title.trim() || submitting}
          className="rounded bg-indigo-600 px-3 py-1 text-sm text-white hover:bg-indigo-700 disabled:opacity-50"
          data-drag="false"
        >
          Добавить
        </button>
      </div>
    </div>
  );
}
