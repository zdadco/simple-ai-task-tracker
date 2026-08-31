import { useState, type ReactNode } from "react";
import {
  PRIORITY_COLORS,
  PRIORITY_LABELS,
  deleteTask,
  enqueueAnalysis,
  formatDate,
  updateTask,
  type Task,
  type TaskPriority,
} from "../lib/tauri";
import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";

interface TaskItemProps {
  task: Task;
  onUpdated: () => void;
}

const ANALYSIS_CHIP: Record<string, { label: string; className: string }> = {
  pending: { label: "Ожидание", className: "bg-slate-100 text-slate-600" },
  running: { label: "Анализ…", className: "bg-indigo-50 text-indigo-700" },
  done: { label: "✓ Анализ", className: "bg-emerald-50 text-emerald-700" },
  failed: { label: "Ошибка анализа", className: "bg-red-50 text-red-700" },
};

function IconButton({
  label,
  onClick,
  disabled,
  danger,
  children,
}: {
  label: string;
  onClick: () => void;
  disabled?: boolean;
  danger?: boolean;
  children: ReactNode;
}) {
  return (
    <button
      type="button"
      title={label}
      aria-label={label}
      onClick={onClick}
      disabled={disabled}
      className={`rounded-lg p-1.5 disabled:opacity-40 ${
        danger
          ? "text-gray-400 hover:bg-red-50 hover:text-red-600"
          : "text-gray-400 hover:bg-gray-100 hover:text-gray-700"
      }`}
    >
      {children}
    </button>
  );
}

export default function TaskItem({ task, onUpdated }: TaskItemProps) {
  const [editing, setEditing] = useState(false);
  const [title, setTitle] = useState(task.title);
  const [notesOpen, setNotesOpen] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [deleting, setDeleting] = useState(false);
  const [deleteError, setDeleteError] = useState<string | null>(null);

  const { attributes, listeners, setNodeRef, transform, transition, isDragging } =
    useSortable({ id: task.id });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  const done = task.status === "done";
  const analysis = ANALYSIS_CHIP[task.analysisStatus];

  async function handleSave() {
    if (title.trim() && title !== task.title) {
      await updateTask(task.id, title.trim());
      onUpdated();
    }
    setEditing(false);
  }

  async function handlePriorityChange(priority: TaskPriority) {
    await updateTask(task.id, undefined, priority);
    onUpdated();
  }

  async function handleToggleDone() {
    await updateTask(
      task.id,
      undefined,
      undefined,
      done ? "open" : "done",
    );
    onUpdated();
  }

  async function handleDelete() {
    setDeleteError(null);
    setDeleting(true);
    try {
      await deleteTask(task.id);
      setConfirmDelete(false);
      onUpdated();
    } catch (e) {
      setDeleteError(String(e));
    } finally {
      setDeleting(false);
    }
  }

  async function handleReanalyze() {
    await enqueueAnalysis(task.id);
    onUpdated();
  }

  return (
    <div
      ref={setNodeRef}
      style={style}
      className={`rounded-2xl bg-white p-4 shadow-sm ring-1 ring-gray-200/60 ${
        done ? "opacity-70" : ""
      }`}
    >
      <div className="flex items-start gap-3">
        <input
          type="checkbox"
          checked={done}
          onChange={handleToggleDone}
          className="mt-1.5 h-4 w-4 rounded border-gray-300"
          title={done ? "Вернуть в открытые" : "Выполнено"}
        />
        <button
          type="button"
          className="mt-1 cursor-grab text-gray-300 hover:text-gray-500 active:cursor-grabbing"
          {...attributes}
          {...listeners}
          aria-label="Перетащить"
        >
          ⠿
        </button>

        <div className="min-w-0 flex-1">
          {editing ? (
            <div className="flex gap-2">
              <input
                className="flex-1 rounded-lg border border-gray-300 px-2.5 py-1.5 text-sm"
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") handleSave();
                  if (e.key === "Escape") {
                    setTitle(task.title);
                    setEditing(false);
                  }
                }}
                autoFocus
              />
              <button
                type="button"
                onClick={handleSave}
                className="rounded-lg bg-indigo-600 px-2.5 py-1.5 text-xs text-white hover:bg-indigo-700"
              >
                OK
              </button>
            </div>
          ) : (
            <p
              className={`text-base font-medium leading-snug text-gray-900 ${
                done ? "line-through text-gray-500" : ""
              }`}
            >
              {task.title}
            </p>
          )}

          <div className="mt-2.5 flex flex-wrap items-center gap-1.5">
            <label className="relative inline-flex">
              <span className="sr-only">Приоритет</span>
              <select
                value={task.priority}
                onChange={(e) =>
                  handlePriorityChange(e.target.value as TaskPriority)
                }
                className={`cursor-pointer appearance-none rounded-md border-0 py-0.5 pl-2 pr-6 text-xs font-medium ${PRIORITY_COLORS[task.priority]}`}
              >
                {(Object.keys(PRIORITY_LABELS) as TaskPriority[]).map((p) => (
                  <option key={p} value={p}>
                    {PRIORITY_LABELS[p]}
                  </option>
                ))}
              </select>
              <span className="pointer-events-none absolute right-1.5 top-1/2 -translate-y-1/2 text-[10px] opacity-60">
                ▾
              </span>
            </label>

            <span className="rounded-md bg-gray-100 px-2 py-0.5 text-xs text-gray-600">
              {formatDate(task.createdAt)}
            </span>

            {analysis && (
              <span
                className={`rounded-md px-2 py-0.5 text-xs font-medium ${analysis.className}`}
              >
                {analysis.label}
              </span>
            )}

            {task.agentNotes && (
              <button
                type="button"
                onClick={() => setNotesOpen(!notesOpen)}
                className="rounded-md bg-indigo-50 px-2 py-0.5 text-xs font-medium text-indigo-700 hover:bg-indigo-100"
              >
                {notesOpen ? "Скрыть заметки" : "Заметки агента"}
              </button>
            )}
          </div>

          {notesOpen && task.agentNotes && (
            <pre className="mt-2 whitespace-pre-wrap rounded-xl bg-gray-50 p-2.5 text-xs text-gray-700 ring-1 ring-gray-100">
              {task.agentNotes}
            </pre>
          )}

          {confirmDelete && (
            <div className="mt-2 flex flex-wrap items-center gap-2 text-xs">
              <span className="text-gray-500">Удалить задачу?</span>
              <button
                type="button"
                onClick={handleDelete}
                disabled={deleting}
                className="rounded-md bg-red-600 px-2 py-1 text-white hover:bg-red-700 disabled:opacity-50"
              >
                {deleting ? "…" : "Удалить"}
              </button>
              <button
                type="button"
                onClick={() => {
                  setConfirmDelete(false);
                  setDeleteError(null);
                }}
                disabled={deleting}
                className="rounded-md px-2 py-1 text-gray-600 hover:bg-gray-100"
              >
                Отмена
              </button>
              {deleteError && (
                <span className="text-red-600">{deleteError}</span>
              )}
            </div>
          )}
        </div>

        <div className="flex shrink-0 gap-0.5">
          <IconButton label="Изменить" onClick={() => setEditing(true)}>
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden>
              <path
                d="M11.5 2.5l2 2L5 13H3v-2L11.5 2.5z"
                stroke="currentColor"
                strokeWidth="1.4"
                strokeLinejoin="round"
              />
            </svg>
          </IconButton>
          <IconButton
            label="Анализ"
            onClick={handleReanalyze}
            disabled={task.analysisStatus === "running"}
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden>
              <path
                d="M13 8A5 5 0 1 1 8 3"
                stroke="currentColor"
                strokeWidth="1.4"
                strokeLinecap="round"
              />
              <path
                d="M8 1.5V4l2-1"
                stroke="currentColor"
                strokeWidth="1.4"
                strokeLinecap="round"
                strokeLinejoin="round"
              />
            </svg>
          </IconButton>
          <IconButton
            label="Удалить"
            danger
            onClick={() => setConfirmDelete(true)}
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden>
              <path
                d="M3.5 4.5h9M6 4.5V3.5h4v1M6.5 7v4.5M9.5 7v4.5M4.5 4.5l.5 8.5h6l.5-8.5"
                stroke="currentColor"
                strokeWidth="1.4"
                strokeLinecap="round"
                strokeLinejoin="round"
              />
            </svg>
          </IconButton>
        </div>
      </div>
    </div>
  );
}
