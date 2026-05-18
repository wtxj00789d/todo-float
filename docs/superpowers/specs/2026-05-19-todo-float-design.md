# Todo Float Design

## Goal

Build a lightweight Windows-first desktop todo utility that shows today's todo list on login only when there is something relevant to show. The app should feel like a restrained Win11/Codex-style floating panel, avoid slowing startup, and use an LLM only when the user actively adds natural-language todos.

The first version targets Windows 11 while keeping the core data and parsing boundaries portable enough for later cross-platform work.

## Product Shape

- Desktop app built with Tauri.
- Portable-first distribution for v1: unzip/run, with `config.toml` beside the executable.
- No settings page in v1.
- Default window is a narrow floating panel around 300 px wide, with long content wrapping and the panel allowed to grow up to about 480 px only when necessary.
- Visual direction follows Win11 acrylic/mica and the current Codex floating panel: restrained, low chrome, compact spacing, and system theme by default with dark mode supported.
- Todo items are displayed as a numbered list. No completion checkbox is shown in v1.
- The user can keep the float open after launching it normally, then manually minimize or close it.

Mockups created during design:

- `mockups/todo-float-v1.html`
- `mockups/todo-float-v2.html`
- `mockups/todo-float-v3.html`

## Runtime Modes

The app has two launch modes.

### startup-check

Triggered by Windows login startup. This mode must stay very fast:

1. Open local SQLite.
2. Read the current local system date.
3. Query today's active todos.
4. Check whether today's relevant todo set has already been shown.
5. If no popup is needed, exit without creating a window.
6. If a popup is needed, create the floating window.

This mode never calls an LLM.

### normal

Launched by the user from a shortcut/start menu. This opens the float so the user can view today's list and add natural-language todos.

## Popup Rules

- Use the computer's current local date at login time. Do not infer a new day from elapsed hours.
- If today's active todo list is empty, exit silently.
- If today's todos have already been shown today and no new today's todo was added afterward, exit silently.
- If a new todo due today was added after the last popup record, show the popup on the next startup check.
- "Do not show again today" suppresses the current todo set only. If new today-due todos are added later, they can trigger the popup again on the next startup.
- A machine left on past midnight does not trigger anything by itself. The next login/startup check uses that day's date.

Example: if the machine starts on 2026-05-18 at 09:00, shuts down on 2026-05-19 at 02:00, and starts again on 2026-05-19 at 09:00, the 2026-05-19 startup check is the first check for that date and should show 2026-05-19 todos when present.

## Data Model

Use SQLite.

### todos

- `id`: primary key.
- `title`: todo text.
- `due_date`: required `YYYY-MM-DD`.
- `due_time`: nullable `HH:mm`; reserved for future time-aware features.
- `source_text`: original user input.
- `date_expression`: extracted phrase such as `明天` or `下周二`.
- `date_source`: `rule`, `llm`, or `default`.
- `warning`: nullable user-visible warning, such as a past-date warning.
- `created_at`: timestamp.
- `updated_at`: timestamp.
- `deleted_at`: nullable timestamp for soft delete.
- `completed_at`: nullable timestamp, reserved but not surfaced in v1 UI.

### popup_events

- `id`: primary key.
- `date`: local date for the popup event.
- `shown_at`: timestamp.
- `todo_snapshot_max_created_at`: max `created_at` among active todos due on that date when shown.
- `suppressed`: boolean for "do not show again today" on the current snapshot.

The startup check compares today's active todos against the latest popup event for the same date. If the max todo creation time is newer than the recorded snapshot, the popup can show again.

## Config

V1 uses `config.toml` beside the executable. There is no settings UI.

```toml
[llm]
provider = "openrouter"
api_key = ""
model = "z-ai/glm-4.5-air"

[llm.fallback]
provider = "bigmodel"
api_key = ""
model = "glm-4.7-flash"
```

If the configured key is empty, natural-language parsing shows a local prompt telling the user to fill the config file. No API key is hard-coded or stored in the design document.

If v2 becomes a formal installer under `Program Files`, config moves to a user-writable location such as `%APPDATA%`.

## LLM Providers

V1 includes only two providers:

- Default: OpenRouter with `z-ai/glm-4.5-air`.
- Fallback: BigModel direct with `glm-4.7-flash`.

Testing notes:

- OpenRouter GLM-4.5 Air passed the 18-case enhanced prompt batch in testing.
- BigModel direct GLM-4.7 Flash connected successfully and returned valid JSON, but showed weaker date and item-splitting accuracy in the same enhanced batch.
- SiliconFlow DeepSeek R1 Qwen3 8B was tested but is excluded from v1 because date handling and item splitting were less stable.

## Parsing Flow

Natural-language parsing runs only after the user actively enters text.

1. Read config and choose the primary provider.
2. Send the enhanced prompt with current date, weekday, timezone, and explicit current-week/next-week date tables.
3. Require JSON output with entries.
4. Parse JSON.
5. Validate schema.
6. Run local deterministic date rules against the original input and extracted date expressions.
7. If a local rule determines the date, override the LLM date and set `date_source = "rule"`.
8. If no local rule matches but the LLM date is valid, use it and set `date_source = "llm"`.
9. If no usable date exists, default to the current date or require manual correction and set `date_source = "default"`.
10. Show a preview for user confirmation.
11. Save only after confirmation.

The preview UI lets the user edit each proposed item title and due date before saving. This is the only correction path in v1; there is no separate settings or advanced editor.

## Local Date Rules

Local rules are the final authority for common expressions:

- `今天`
- `明天`
- `后天`
- `大后天`
- `本周`, `这周`, `这个周`
- `下周`, `下星期`, `下礼拜`
- `下下周`, `下下星期`, `下下礼拜`
- Bare weekday expressions such as `周二`, `星期二`, `礼拜二`
- `周五之前`, `周五前`
- `月底`
- `月初`
- `下个月3号`, `下个月三号`
- Month-day expressions such as `6月1号`
- `下周末`, defaulting to Saturday unless the user explicitly says Sunday

Week calculation follows the user's habit: Monday is the first day of the week and Sunday is the last.

Bare weekday expressions refer to the current week. If the resulting date is already in the past, keep that date and show a warning in preview.

## Validation And Errors

The app validates two layers:

- JSON validity: the model response must be parseable JSON.
- Schema validity: entries must exist, each entry must have a valid `due_date` and non-empty item text.

Failure handling:

- Missing config key: do not call the provider; tell the user to fill the config file.
- Primary provider failure: try the configured fallback when available.
- Fallback failure: show a manual entry/edit flow.
- Invalid JSON: do not save automatically.
- Schema mismatch: do not save automatically.
- Local rule and LLM conflict: local rule wins and preview shows a warning.
- SQLite read failure during startup check: exit without blocking startup.
- SQLite write failure during normal use: show an error.

## First Version Scope

Included:

- Tauri app scaffold.
- SQLite local storage.
- Config file next to executable.
- Startup-check mode.
- Normal floating window mode.
- Today's todo popup.
- Natural-language add flow with preview confirmation.
- OpenRouter default provider and BigModel fallback.
- Local date rule validation and override.

Not included:

- Settings UI.
- Cloud sync.
- Account system.
- Tray resident process.
- Global hotkey.
- Completion checkbox UI.
- Exact-time reminders.
- Mobile or browser PWA.
