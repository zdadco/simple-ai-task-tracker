# Micro Task Tracker

Десктопное tray-приложение для быстрого захвата задач с фоновым LLM-анализом.

## Возможности

- **Tray-приложение** — работает в фоне, окно скрыто при старте
- **Быстрый ввод** — глобальная горячая клавиша `Ctrl+Shift+T` (настраивается)
- **Drag-and-drop** — изменение порядка задач (выше = важнее)
- **Приоритеты** — low / medium / high / critical с фильтрацией
- **LLM-анализ** — OpenAI-compatible API (OpenAI, Groq, Ollama, LM Studio)
- **Автозапуск** — включение/выключение в настройках
- **Голосовой ввод** — через встроенную диктовку ОС (без платных STT API)

## Требования

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://www.rust-lang.org/tools/install) (stable)
- Зависимости Tauri 2 для Windows: [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)

## Установка и запуск

```bash
npm install
npm run tauri:dev
```

На Windows, если `cargo` не находится в терминале Cursor, скрипт `tauri:dev` добавляет `%USERPROFILE%\.cargo\bin` в PATH автоматически.

## Сборка

```bash
npm run tauri:build
```

Артефакты появятся в `src-tauri/target/release/bundle/`.

## GitHub Releases

Workflow [`.github/workflows/release.yml`](.github/workflows/release.yml) собирает установщики для Windows, macOS (Intel + Apple Silicon) и Linux и создаёт **draft** release.

1. Синхронизируйте версию в `package.json` и `src-tauri/tauri.conf.json` (сейчас `0.1.0`).
2. Запушьте тег:

```bash
git tag v0.1.0
git push origin v0.1.0
```

Либо запустите workflow вручную: **Actions → Release → Run workflow**.

3. После сборки откройте draft на вкладке Releases, проверьте assets и нажмите **Publish release**.

> Подпись/нотаризация macOS и подпись Windows не настроены — для публичной раздачи позже можно добавить secrets (`APPLE_*`, Windows certificate).

## Горячие клавиши

| Действие | Клавиши |
|----------|---------|
| Быстрый ввод задачи | `Ctrl+Shift+T` (по умолчанию) |
| Отправить задачу | `Enter` в окне Quick Capture |
| Закрыть Quick Capture | `Esc` |

## Голосовой ввод (ОС)

Приложение **не** вызывает Whisper или другие STT API. Используйте системную диктовку:

| ОС | Шорткат |
|----|---------|
| **Windows** | `Win+H` — голосовой ввод Windows |
| **macOS** | `Fn+Fn` или двойной `Control` — диктовка |
| **Linux** | Зависит от DE (GNOME/KDE speech-to-text) |

Откройте Quick Capture (`Ctrl+Shift+T`), сфокусируйтесь на поле ввода и активируйте диктовку ОС.

## Настройка LLM

### Ollama (локально)

1. Установите [Ollama](https://ollama.com/)
2. Запустите модель: `ollama pull llama3.2`
3. В настройках приложения:
   - **Base URL:** `http://localhost:11434/v1`
   - **API Key:** *(оставьте пустым)*
   - **Model:** `llama3.2`

### OpenAI / Groq

- **OpenAI:** `https://api.openai.com/v1`, API key, модель `gpt-4o-mini`
- **Groq:** `https://api.groq.com/openai/v1`, API key, модель `llama-3.3-70b-versatile`

## Хранение данных

SQLite база: `%APPDATA%\simple-ai-task-tracker\tasks.db` (Windows).

На macOS: `~/Library/Application Support/simple-ai-task-tracker/tasks.db`  
На Linux: `~/.local/share/simple-ai-task-tracker/tasks.db`

## Структура проекта

**Bundle identifier:** `ru.zdadco.mtt` (Micro Task Tracker)

```
simple-ai-task-tracker/
├── src/                    # React frontend
│   ├── windows/            # QuickCapture, TaskList, Settings
│   ├── components/         # TaskItem, SortableTaskList, PriorityFilter
│   └── lib/tauri.ts        # invoke wrappers
├── src-tauri/              # Rust backend
│   └── src/
│       ├── db/             # SQLite schema, tasks CRUD
│       ├── agent/          # LLM client, background worker
│       ├── commands/       # Tauri commands
│       └── tray.rs         # System tray
└── README.md
```

## Tray-меню

- **Новая задача** — открыть Quick Capture
- **Открыть список** — главное окно с задачами
- **Настройки** — LLM, hotkey, автозапуск
- **Выход**

## Лицензия

MIT

## Port 1420 already in use

`npm run tauri:dev` starts Vite on port **1420** (`strictPort: true`). If a previous dev session did not exit cleanly, you may see `Port 1420 is already in use`.

- **Automatic:** `pretauri:dev` runs `scripts/kill-port.mjs` before `tauri:dev` and stops any process listening on 1420.
- **Manual:** `npm run kill-port` or `netstat -ano | findstr :1420` then `taskkill /PID <pid> /F` (Windows). Also close orphan `simple-ai-task-tracker.exe` from `src-tauri/target/debug`.

## Cursor / VS Code terminal (Windows)

If `cargo metadata: program not found` appears in the integrated terminal even though Rust is installed, use:

```bash
npm run tauri:dev
```

The `scripts/run-tauri.mjs` wrapper adds `%USERPROFILE%\.cargo\bin` to `PATH` for Tauri. The project also sets `terminal.integrated.env.windows` in `.vscode/settings.json` so new Cursor terminals include Cargo on `PATH`.
