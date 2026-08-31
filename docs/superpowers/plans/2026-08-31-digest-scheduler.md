# Digests Scheduler Implementation Plan

> **For agentic workers:** Implement task-by-task. Steps use checkbox syntax.

**Goal:** Scheduled daily/weekly/monthly digests with custom prompts/times, task done status, Digests UI, OS notifications, local LLM fallback + AI pin.

**Architecture:** Rust `DigestScheduler` (60s tick, local TZ) writes `digests` rows; generator uses LLM or local composer; React Digests window + Settings; `tauri-plugin-notification`.

**Tech Stack:** Tauri 2, rusqlite, chrono, tokio, React, tauri-plugin-notification

## Global Constraints

- Preview ≤ 160 chars; week starts Monday; catch-up max 1 per current period per kind
- Task digest filter: `open` + created in `[period_start, period_end)`
- AI pin when `source === llm`

---

### Task 1: Schema + task status + digests DB

- Migrate `tasks.status`, table `digests`, settings keys
- CRUD digests; list open tasks in period; update task status

### Task 2: Period math + scheduler + generator

- `digest/period.rs`, `generator.rs`, `scheduler.rs`
- Notifications plugin; commands `list_digests`, `generate_digest_now`, `get_digest`

### Task 3: Frontend

- Digests window; Settings digest section; task done UI; tray/menu

### Task 4: Verify

- `cargo check`, `npm run build`, commit
