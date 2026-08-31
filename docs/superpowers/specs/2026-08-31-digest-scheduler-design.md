# Digests scheduler — Design

**Date:** 2026-08-31  
**Branch:** `feat/digest-scheduler`  
**Status:** Approved — implementing

## Problem

Users need periodic AI (or local) summaries of unfinished work: start of day, week, and month. Schedules and prompts must be customizable; times use the system local timezone. Results appear in a **Дайджесты** section and as OS notifications.

## Goals

- Scheduled digests: **daily**, **weekly**, **monthly**
- Custom prompt + custom `HH:MM` per kind (local TZ)
- Task **status** (`open` / `done`); digest input = open tasks **created in the period**
- Catch-up: at most **one** digest per current period per kind (no backfill of many past days)
- OS notification with **`preview` ≤ 160 chars**; full text in Digests UI
- Without LLM: still create a **local** digest from DB; LLM digests show an **AI** pin in UI
- App remains tray-first; scheduler runs in Rust backend

## Non-goals (v1)

- Due dates / calendars
- Email or external calendar sync
- Signed notification deep-link payloads beyond opening Digests
- Generating digests for past completed periods (only current period catch-up)

## Data model

### Tasks

Add `status TEXT NOT NULL DEFAULT 'open'` with values `open` | `done`.

UI: mark done / reopen; list filter all / open / done (default: open).

### Table `digests`

| Column | Type | Notes |
|--------|------|--------|
| `id` | TEXT PK | UUID |
| `kind` | TEXT | `daily` \| `weekly` \| `monthly` |
| `period_start` | INTEGER | unix, local period start |
| `period_end` | INTEGER | unix, exclusive or inclusive end (document in impl) |
| `content` | TEXT | full markdown / plain text |
| `preview` | TEXT | ≤ 160 chars for notifications |
| `source` | TEXT | `llm` \| `local` |
| `status` | TEXT | `pending` \| `running` \| `done` \| `failed` |
| `error` | TEXT NULL | on failure |
| `created_at` | INTEGER | unix |

Unique constraint: `(kind, period_start)` — enforces one digest per period.

### Settings (KV)

Per kind (`daily` / `weekly` / `monthly`):

- `{kind}_enabled` (bool, default true)
- `{kind}_time` (`HH:MM`, default `09:00`)
- `{kind}_prompt_template` (string)

Shared placeholders for prompts: `{kind}`, `{period_start}`, `{period_end}`, `{tasks}`.

Week starts **Monday**. Monthly runs on the **1st** at configured time.

## Period rules

Computed in **system local timezone**:

| Kind | Period |
|------|--------|
| daily | calendar day containing “now” |
| weekly | Monday 00:00 → next Monday 00:00 (or end of Sunday) |
| monthly | 1st 00:00 → 1st of next month |

**Task selection:** `status = open` AND `created_at` ∈ `[period_start, period_end)`.

## Scheduler

- Spawned in app `setup` (tokio), tick ~every **60s**.
- For each enabled kind:
  1. Compute current period bounds and scheduled fire time for that period.
  2. If local now ≥ fire time for this period AND no row for `(kind, period_start)` → enqueue generation.
- **Catch-up:** same rule — only the **current** period, max one digest; does not create digests for previous days/weeks/months.
- Daily/weekly/monthly all catch up under this rule.

## Generation pipeline

1. Insert digest `pending` → `running`.
2. Load matching tasks.
3. If LLM configured and reachable (reuse existing connection check / chat call):
   - Render prompt with `{tasks}` list.
   - Call OpenAI-compatible API.
   - Save `content`, derive `preview` (truncate ≤ 160 + ellipsis), `source = llm`, `status = done`.
4. Else **local fallback**:
   - Build text: count of open tasks in period; highest urgency task (`critical` > `high` > `medium` > `low`, then `sort_order`); optional top-3 titles.
   - If zero tasks: fixed message «Нет незавершённых задач за период».
   - `source = local`, `preview` truncated.
5. On LLM failure after attempting: prefer local fallback rather than only `failed` (recommended); if both fail, `failed` + error notification.
6. Emit Tauri event `digest-updated`; show OS notification.

## Notifications

- Plugin: `tauri-plugin-notification`.
- Success: title by kind («Дайджест дня/недели/месяца»), body = `preview`.
- Failure: «Ошибка дайджеста» + short error.
- Click: show Digests window (and select digest if feasible).

## UI

### Task list

- Done toggle; filter open/done/all.

### Digests section

- Tray / header entry «Дайджесты».
- List: kind, period label, status, `preview`; **AI pin** when `source === 'llm'`.
- Detail: full `content`.
- «Сгенерировать сейчас» per kind (uses current period; respects unique constraint — regenerate = update or replace with user confirm; **v1: if exists, regenerate in place**).

### Settings

Section «Дайджесты»: enable, time, prompt textarea ×3 kinds.

## Architecture sketch

```text
lib.rs setup
  ├── AgentWorker (existing per-task analysis)
  └── DigestScheduler ──► DigestGenerator ──► LlmClient | LocalComposer
                              │
                              ├── SQLite digests
                              ├── emit digest-updated
                              └── notification plugin
```

## Error handling

- Missing LLM → local digest (not a hard failure).
- Empty task set → local empty message.
- Duplicate period → skip (unique constraint).
- Clock skew / TZ change → next tick re-evaluates local now.

## Testing (manual)

- [ ] Toggle task done; digest query excludes done.
- [ ] Set daily time to near-now; receive notification + Digests row with AI pin if LLM ok.
- [ ] Disable LLM / wrong URL → local digest, no AI pin.
- [ ] Catch-up: set time in the past same day, restart app → one daily digest, not many.
- [ ] Weekly only fires Monday after time; monthly on 1st.
- [ ] Preview ≤ 160 in notification.

## Open implementation notes

- Prefer regenerating in place for manual «Сгенерировать сейчас».
- Prefer LLM failure → local fallback before marking `failed`.
