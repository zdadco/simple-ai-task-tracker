import type { TaskPriority } from "../lib/tauri";

interface PriorityFilterProps {
  value: TaskPriority | "all";
  onChange: (value: TaskPriority | "all") => void;
}

const OPTIONS: { value: TaskPriority | "all"; label: string }[] = [
  { value: "all", label: "Все" },
  { value: "critical", label: "Критический" },
  { value: "high", label: "Высокий" },
  { value: "medium", label: "Средний" },
  { value: "low", label: "Низкий" },
];

export default function PriorityFilter({ value, onChange }: PriorityFilterProps) {
  return (
    <div className="flex flex-wrap gap-2">
      {OPTIONS.map((opt) => (
        <button
          key={opt.value}
          type="button"
          onClick={() => onChange(opt.value)}
          className={`rounded-full px-3 py-1 text-sm transition-colors ${
            value === opt.value
              ? "bg-indigo-600 text-white"
              : "bg-gray-100 text-gray-700 hover:bg-gray-200"
          }`}
        >
          {opt.label}
        </button>
      ))}
    </div>
  );
}
