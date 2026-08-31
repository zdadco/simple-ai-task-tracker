import { showQuickCapture } from "../lib/tauri";
import type { AppRoute } from "../lib/nav";

interface AppNavProps {
  route: AppRoute;
  onNavigate: (route: AppRoute) => void;
}

const TITLES: Record<AppRoute, string> = {
  tasks: "Задачи",
  digests: "Дайджесты",
  settings: "Настройки",
};

export default function AppNav({ route, onNavigate }: AppNavProps) {
  return (
    <header className="flex items-center justify-between border-b border-gray-200 bg-white px-6 py-4">
      <h1 className="text-lg font-semibold text-gray-900">{TITLES[route]}</h1>
      <div className="flex flex-wrap gap-2">
        <button
          type="button"
          onClick={() => showQuickCapture()}
          className="rounded-lg bg-indigo-600 px-3 py-1.5 text-sm text-white hover:bg-indigo-700"
        >
          + Новая
        </button>
        {(
          [
            ["tasks", "Задачи"],
            ["digests", "Дайджесты"],
            ["settings", "Настройки"],
          ] as const
        ).map(([id, label]) => (
          <button
            key={id}
            type="button"
            onClick={() => onNavigate(id)}
            className={`rounded-lg px-3 py-1.5 text-sm ${
              route === id
                ? "bg-indigo-50 text-indigo-700 ring-1 ring-indigo-200"
                : "border border-gray-200 text-gray-700 hover:bg-gray-50"
            }`}
          >
            {label}
          </button>
        ))}
      </div>
    </header>
  );
}
