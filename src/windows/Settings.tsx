import { useEffect, useState } from "react";
import {
  getSettings,
  saveSettings,
  testLlmConnection,
  type AppSettings,
} from "../lib/tauri";

const DEFAULT_PROMPT = `Проанализируй задачу и дай краткие заметки (3–5 пунктов):
что важно, возможные шаги, риски.

Задача: {title}
Приоритет: {priority}
Создана: {created_at}

Ответь на русском, в markdown.`;

export default function Settings() {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [saving, setSaving] = useState(false);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);

  useEffect(() => {
    getSettings().then(setSettings);
  }, []);

  if (!settings) {
    return <div className="p-6 text-gray-400">Загрузка...</div>;
  }

  function update(partial: Partial<AppSettings>) {
    setSettings((s) => (s ? { ...s, ...partial } : s));
  }

  async function handleSave() {
    if (!settings) return;
    setSaving(true);
    setMessage(null);
    try {
      await saveSettings(settings);
      setMessage("Настройки сохранены");
    } catch (e) {
      setMessage(`Ошибка: ${e}`);
    } finally {
      setSaving(false);
    }
  }

  async function handleTest() {
    if (!settings) return;
    setTesting(true);
    setTestResult(null);
    try {
      const result = await testLlmConnection(
        settings.llmBaseUrl,
        settings.llmApiKey,
        settings.llmModel,
      );
      setTestResult(`OK: ${result.slice(0, 100)}`);
    } catch (e) {
      setTestResult(`Ошибка: ${e}`);
    } finally {
      setTesting(false);
    }
  }

  return (
    <div className="h-screen overflow-y-auto bg-gray-50 p-6">
      <h1 className="mb-6 text-xl font-semibold text-gray-900">Настройки</h1>

      <section className="mb-6 space-y-4 rounded-lg bg-white p-4 shadow-sm">
        <h2 className="font-medium text-gray-800">LLM (OpenAI-compatible)</h2>

        <label className="block">
          <span className="text-sm text-gray-600">Base URL</span>
          <input
            type="url"
            value={settings.llmBaseUrl}
            onChange={(e) => update({ llmBaseUrl: e.target.value })}
            placeholder="http://localhost:11434/v1"
            className="mt-1 w-full rounded border border-gray-200 px-3 py-2 text-sm"
          />
        </label>

        <label className="block">
          <span className="text-sm text-gray-600">API Key (оставьте пустым для Ollama)</span>
          <input
            type="password"
            value={settings.llmApiKey}
            onChange={(e) => update({ llmApiKey: e.target.value })}
            className="mt-1 w-full rounded border border-gray-200 px-3 py-2 text-sm"
          />
        </label>

        <label className="block">
          <span className="text-sm text-gray-600">Model</span>
          <input
            type="text"
            value={settings.llmModel}
            onChange={(e) => update({ llmModel: e.target.value })}
            placeholder="llama3.2"
            className="mt-1 w-full rounded border border-gray-200 px-3 py-2 text-sm"
          />
        </label>

        <div className="flex gap-2">
          <button
            type="button"
            onClick={handleTest}
            disabled={testing}
            className="rounded border border-gray-200 px-3 py-1.5 text-sm hover:bg-gray-50 disabled:opacity-50"
          >
            {testing ? "Проверка..." : "Тест подключения"}
          </button>
          {testResult && (
            <span className="self-center text-xs text-gray-500">{testResult}</span>
          )}
        </div>
      </section>

      <section className="mb-6 space-y-4 rounded-lg bg-white p-4 shadow-sm">
        <h2 className="font-medium text-gray-800">Промпт агента</h2>
        <p className="text-xs text-gray-500">
          Плейсхолдеры: {"{title}"}, {"{priority}"}, {"{created_at}"}
        </p>
        <textarea
          value={settings.agentPromptTemplate}
          onChange={(e) => update({ agentPromptTemplate: e.target.value })}
          rows={8}
          className="w-full rounded border border-gray-200 px-3 py-2 font-mono text-sm"
        />
        <button
          type="button"
          onClick={() => update({ agentPromptTemplate: DEFAULT_PROMPT })}
          className="text-xs text-indigo-600 hover:underline"
        >
          Сбросить к шаблону по умолчанию
        </button>

        <label className="flex items-center gap-2">
          <input
            type="checkbox"
            checked={settings.analyzeOnCreate}
            onChange={(e) => update({ analyzeOnCreate: e.target.checked })}
          />
          <span className="text-sm">Анализировать при создании задачи</span>
        </label>
      </section>

      <section className="mb-6 space-y-4 rounded-lg bg-white p-4 shadow-sm">
        <h2 className="font-medium text-gray-800">Горячая клавиша</h2>
        <label className="block">
          <span className="text-sm text-gray-600">
            Глобальный шорткат (например Ctrl+Shift+T)
          </span>
          <input
            type="text"
            value={settings.globalHotkey}
            onChange={(e) => update({ globalHotkey: e.target.value })}
            className="mt-1 w-full rounded border border-gray-200 px-3 py-2 text-sm"
          />
        </label>
        <p className="text-xs text-gray-400">
          Формат: модификаторы через + (Ctrl, Shift, Alt, Win) и буква/клавиша.
        </p>
      </section>

      <section className="mb-6 space-y-4 rounded-lg bg-white p-4 shadow-sm">
        <h2 className="font-medium text-gray-800">Автозапуск</h2>
        <label className="flex items-center gap-2">
          <input
            type="checkbox"
            checked={settings.autostartEnabled}
            onChange={(e) => update({ autostartEnabled: e.target.checked })}
          />
          <span className="text-sm">Запускать при старте системы</span>
        </label>
      </section>

      <section className="mb-6 rounded-lg bg-white p-4 shadow-sm">
        <h2 className="mb-2 font-medium text-gray-800">Голосовой ввод (ОС)</h2>
        <ul className="space-y-1 text-sm text-gray-600">
          <li>
            <strong>Windows:</strong> Win+H — голосовой ввод Windows
          </li>
          <li>
            <strong>macOS:</strong> Fn+Fn или двойной Control — диктовка
          </li>
          <li>
            <strong>Linux:</strong> зависит от DE (GNOME/KDE speech-to-text)
          </li>
        </ul>
        <p className="mt-2 text-xs text-gray-400">
          Приложение не использует платные STT API — только системную диктовку.
        </p>
      </section>

      <div className="flex items-center gap-4">
        <button
          type="button"
          onClick={handleSave}
          disabled={saving}
          className="rounded-lg bg-indigo-600 px-4 py-2 text-sm text-white hover:bg-indigo-700 disabled:opacity-50"
        >
          {saving ? "Сохранение..." : "Сохранить"}
        </button>
        {message && <span className="text-sm text-gray-500">{message}</span>}
      </div>
    </div>
  );
}
