import { useState } from "react";
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

export default function TaskItem({ task, onUpdated }: TaskItemProps) {
  const [editing, setEditing] = useState(false);
  const [title, setTitle] = useState(task.title);
  const [notesOpen, setNotesOpen] = useState(false);

  const { attributes, listeners, setNodeRef, transform, transition, isDragging } =
    useSortable({ id: task.id });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

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

  async function handleDelete() {
    if (confirm("Удалить задачу?")) {
      await deleteTask(task.id);
      onUpdated();
    }
  }

  async function handleReanalyze() {
    await enqueueAnalysis(task.id);
    onUpdated();
  }

  const statusLabel: Record<string, string> = {
    none: "",
    pending: "⏳ Ожидание",
    running: "🔄 Анализ...",
    done: "✓ Готово",
    failed: "✗ Ошибка",
  };

  return (
    <div
      ref={setNodeRef}
      style={style}
      className="rounded-lg border border-gray-200 bg-white p-4 shadow-sm"
    >
      <div className="flex items-start gap-3">
        <button
          type="button"
          className="mt-1 cursor-grab text-gray-400 hover:text-gray-600 active:cursor-grabbing"
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
                className="flex-1 rounded border border-gray-300 px-2 py-1 text-sm"
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") handleSave();
                  if (e.key === "Escape") setEditing(false);
                }}
                autoFocus
              />
              <button
                type="button"
                onClick={handleSave}
                className="rounded bg-indigo-600 px-2 py-1 text-xs text-white"
              >
                OK
              </button>
            </div>
          ) : (
            <p className="font-medium text-gray-900">{task.title}</p>
          )}

          <div className="mt-2 flex flex-wrap items-center gap-2 text-xs text-gray-500">
            <span>{formatDate(task.createdAt)}</span>
            <span
              className={`rounded-full px-2 py-0.5 font-medium ${PRIORITY_COLORS[task.priority]}`}
            >
              {PRIORITY_LABELS[task.priority]}
            </span>
            {task.analysisStatus !== "none" && (
              <span className="text-gray-400">{statusLabel[task.analysisStatus]}</span>
            )}
          </div>

          {task.agentNotes && (
            <div className="mt-2">
              <button
                type="button"
                onClick={() => setNotesOpen(!notesOpen)}
                className="text-xs text-indigo-600 hover:underline"
              >
                {notesOpen ? "Скрыть заметки агента" : "Показать заметки агента"}
              </button>
              {notesOpen && (
                <pre className="mt-1 whitespace-pre-wrap rounded bg-gray-50 p-2 text-xs text-gray-700">
                  {task.agentNotes}
                </pre>
              )}
            </div>
          )}
        </div>

        <div className="flex flex-col gap-1">
          <select
            value={task.priority}
            onChange={(e) => handlePriorityChange(e.target.value as TaskPriority)}
            className="rounded border border-gray-200 px-1 py-0.5 text-xs"
          >
            <option value="low">Низкий</option>
            <option value="medium">Средний</option>
            <option value="high">Высокий</option>
            <option value="critical">Критический</option>
          </select>
          <button
            type="button"
            onClick={() => setEditing(true)}
            className="text-xs text-gray-500 hover:text-indigo-600"
          >
            Изменить
          </button>
          <button
            type="button"
            onClick={handleReanalyze}
            className="text-xs text-gray-500 hover:text-indigo-600"
            disabled={task.analysisStatus === "running"}
          >
            Анализ
          </button>
          <button
            type="button"
            onClick={handleDelete}
            className="text-xs text-red-500 hover:text-red-700"
          >
            Удалить
          </button>
        </div>
      </div>
    </div>
  );
}
