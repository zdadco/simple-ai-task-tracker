import { invoke } from "@tauri-apps/api/core";

export type TaskPriority = "low" | "medium" | "high" | "critical";
export type TaskStatus = "open" | "done";
export type AnalysisStatus = "none" | "pending" | "running" | "done" | "failed";
export type DigestKind = "daily" | "weekly" | "monthly";

export interface Task {
  id: string;
  title: string;
  createdAt: number;
  updatedAt: number;
  sortOrder: number;
  priority: TaskPriority;
  agentNotes: string | null;
  analysisStatus: AnalysisStatus;
  status: TaskStatus;
}

export interface Digest {
  id: string;
  kind: DigestKind | string;
  periodStart: number;
  periodEnd: number;
  content: string;
  preview: string;
  source: "llm" | "local" | string;
  status: string;
  error: string | null;
  createdAt: number;
}

export interface AppSettings {
  llmBaseUrl: string;
  llmApiKey: string;
  llmModel: string;
  agentPromptTemplate: string;
  analyzeOnCreate: boolean;
  globalHotkey: string;
  autostartEnabled: boolean;
  quickCaptureHintShown: boolean;
  dailyEnabled: boolean;
  dailyTime: string;
  dailyPromptTemplate: string;
  weeklyEnabled: boolean;
  weeklyTime: string;
  weeklyPromptTemplate: string;
  monthlyEnabled: boolean;
  monthlyTime: string;
  monthlyPromptTemplate: string;
}

export async function createTask(title: string): Promise<Task> {
  return invoke("create_task", { title });
}

export async function updateTask(
  id: string,
  title?: string,
  priority?: TaskPriority,
  status?: TaskStatus,
): Promise<Task> {
  return invoke("update_task", { id, title, priority, status });
}

export async function deleteTask(id: string): Promise<void> {
  return invoke("delete_task", { id });
}

export async function listTasks(
  priorityFilter?: string,
  statusFilter?: string,
): Promise<Task[]> {
  return invoke("list_tasks", {
    priorityFilter: priorityFilter ?? null,
    statusFilter: statusFilter ?? null,
  });
}

export async function reorderTasks(orderedIds: string[]): Promise<void> {
  return invoke("reorder_tasks", { orderedIds });
}

export async function getSettings(): Promise<AppSettings> {
  return invoke("get_settings");
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  return invoke("save_settings", { settings });
}

export async function testLlmConnection(
  baseUrl: string,
  apiKey: string,
  model: string,
): Promise<string> {
  return invoke("test_llm_connection", {
    baseUrl,
    apiKey,
    model,
  });
}

export async function enqueueAnalysis(taskId: string): Promise<void> {
  return invoke("enqueue_analysis", { taskId });
}

export async function listDigests(kindFilter?: string): Promise<Digest[]> {
  return invoke("list_digests", { kindFilter: kindFilter ?? null });
}

export async function generateDigestNow(kind: DigestKind): Promise<Digest> {
  return invoke("generate_digest_now", { kind });
}

export async function showQuickCapture(): Promise<void> {
  return invoke("show_quick_capture");
}

export async function showMainWindow(): Promise<void> {
  return invoke("show_main_window");
}

export async function showSettingsWindow(): Promise<void> {
  return invoke("show_settings_window");
}

export async function showDigestsWindow(): Promise<void> {
  return invoke("show_digests_window");
}

export async function hideWindow(label: string): Promise<void> {
  return invoke("hide_window", { label });
}

export async function registerHotkey(): Promise<void> {
  return invoke("register_hotkey");
}

export const PRIORITY_LABELS: Record<TaskPriority, string> = {
  low: "Низкий",
  medium: "Средний",
  high: "Высокий",
  critical: "Критический",
};

export const PRIORITY_COLORS: Record<TaskPriority, string> = {
  low: "bg-slate-100 text-slate-600",
  medium: "bg-blue-100 text-blue-700",
  high: "bg-amber-100 text-amber-800",
  critical: "bg-red-100 text-red-700",
};

export const DIGEST_KIND_LABELS: Record<string, string> = {
  daily: "День",
  weekly: "Неделя",
  monthly: "Месяц",
};

export function formatDate(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleString("ru-RU", {
    day: "2-digit",
    month: "2-digit",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function getDictationHint(): string {
  const platform = navigator.platform.toLowerCase();
  if (platform.includes("win")) {
    return "Голос: Win+H (Windows Speech)";
  }
  if (platform.includes("mac")) {
    return "Голос: Fn+Fn или диктовка macOS (⌃ дважды)";
  }
  return "Голос: системная диктовка вашего окружения рабочего стола";
}
