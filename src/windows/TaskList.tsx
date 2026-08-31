import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import {
  showDigestsWindow,
  showQuickCapture,
  showSettingsWindow,
  listTasks,
  type Task,
  type TaskPriority,
  type TaskStatus,
} from "../lib/tauri";
import PriorityFilter from "../components/PriorityFilter";
import SortableTaskList from "../components/SortableTaskList";

export default function TaskList() {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [filter, setFilter] = useState<TaskPriority | "all">("all");
  const [statusFilter, setStatusFilter] = useState<TaskStatus | "all">("open");
  const [loading, setLoading] = useState(true);

  const loadTasks = useCallback(async () => {
    try {
      const data = await listTasks(
        filter === "all" ? undefined : filter,
        statusFilter === "all" ? undefined : statusFilter,
      );
      setTasks(data);
    } finally {
      setLoading(false);
    }
  }, [filter, statusFilter]);

  useEffect(() => {
    loadTasks();
  }, [loadTasks]);

  useEffect(() => {
    const unlisten = listen("analysis-updated", () => {
      loadTasks();
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [loadTasks]);

  useEffect(() => {
    const interval = setInterval(() => {
      if (tasks.some((t) => t.analysisStatus === "pending" || t.analysisStatus === "running")) {
        loadTasks();
      }
    }, 2000);
    return () => clearInterval(interval);
  }, [tasks, loadTasks]);

  return (
    <div className="flex h-screen flex-col bg-gray-50">
      <header className="flex items-center justify-between border-b border-gray-200 bg-white px-6 py-4">
        <h1 className="text-lg font-semibold text-gray-900">Micro Task Tracker</h1>
        <div className="flex gap-2">
          <button
            type="button"
            onClick={() => showQuickCapture()}
            className="rounded-lg bg-indigo-600 px-3 py-1.5 text-sm text-white hover:bg-indigo-700"
          >
            + Новая
          </button>
          <button
            type="button"
            onClick={() => showDigestsWindow()}
            className="rounded-lg border border-gray-200 px-3 py-1.5 text-sm text-gray-700 hover:bg-gray-50"
          >
            Дайджесты
          </button>
          <button
            type="button"
            onClick={() => showSettingsWindow()}
            className="rounded-lg border border-gray-200 px-3 py-1.5 text-sm text-gray-700 hover:bg-gray-50"
          >
            Настройки
          </button>
        </div>
      </header>

      <div className="space-y-2 border-b border-gray-200 bg-white px-6 py-3">
        <PriorityFilter value={filter} onChange={setFilter} />
        <div className="flex gap-2">
          {(
            [
              ["open", "Открытые"],
              ["done", "Выполненные"],
              ["all", "Все"],
            ] as const
          ).map(([value, label]) => (
            <button
              key={value}
              type="button"
              onClick={() => setStatusFilter(value)}
              className={`rounded-full px-3 py-1 text-xs ${
                statusFilter === value
                  ? "bg-indigo-600 text-white"
                  : "bg-gray-100 text-gray-600 hover:bg-gray-200"
              }`}
            >
              {label}
            </button>
          ))}
        </div>
      </div>

      <main className="flex-1 overflow-y-auto p-6">
        {loading ? (
          <p className="text-center text-gray-400">Загрузка...</p>
        ) : (
          <SortableTaskList tasks={tasks} onUpdated={loadTasks} />
        )}
      </main>
    </div>
  );
}
