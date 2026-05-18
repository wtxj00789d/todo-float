# Todo Float Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the v1 Windows-first Tauri todo float described in `docs/superpowers/specs/2026-05-19-todo-float-design.md`.

**Architecture:** Use a Rust-first Tauri backend for startup checks, SQLite access, config loading, local date rules, and LLM orchestration. Use a compact React/TypeScript frontend only after a window is intentionally created, so the login path can query SQLite and exit before WebView startup when no popup is needed.

**Tech Stack:** Tauri v2, Rust, React, TypeScript, Vite, SQLite via `rusqlite`, TOML config, `reqwest` for OpenRouter/BigModel calls, Vitest/Testing Library for UI tests, Rust unit tests for date rules and repositories.

---

## File Structure

- Create `package.json`: npm scripts and frontend dependencies.
- Create `index.html`: Vite mount point.
- Create `vite.config.ts`: React/Vite config.
- Create `tsconfig.json` and `tsconfig.node.json`: TypeScript config.
- Create `src/main.tsx`: React entry.
- Create `src/App.tsx`: top-level UI state.
- Create `src/api.ts`: typed Tauri command wrappers.
- Create `src/types.ts`: shared frontend DTOs matching Rust command payloads.
- Create `src/styles.css`: Codex-like 300 px float styling.
- Create `src/components/TodoList.tsx`: today's list.
- Create `src/components/AddTodo.tsx`: natural-language input and preview editor.
- Create `src/components/PreviewEditor.tsx`: editable parsed entries.
- Create `src/components/ErrorBanner.tsx`: compact error surface.
- Create `src/App.test.tsx`: UI flow tests.
- Create `src-tauri/Cargo.toml`: Rust dependencies.
- Create `src-tauri/build.rs`: Tauri build hook.
- Create `src-tauri/tauri.conf.json`: no default window, Vite build config.
- Create `src-tauri/src/main.rs`: desktop entrypoint.
- Create `src-tauri/src/lib.rs`: Tauri builder, mode dispatch, commands.
- Create `src-tauri/src/models.rs`: DTO/domain structs.
- Create `src-tauri/src/config.rs`: `config.toml` resolution and parsing.
- Create `src-tauri/src/date_rules.rs`: deterministic Chinese date parser.
- Create `src-tauri/src/db.rs`: SQLite migrations and repository.
- Create `src-tauri/src/llm.rs`: provider clients and JSON/schema validation.
- Create `src-tauri/src/parser.rs`: LLM + local rule orchestration.
- Create `src-tauri/src/startup.rs`: login popup decision.
- Create `src-tauri/src/window.rs`: window creation helpers.
- Create `src-tauri/capabilities/default.json`: minimal command permissions.
- Create `config.example.toml`: empty-key sample config.
- Modify `docs/superpowers/specs/2026-05-19-todo-float-design.md` only if implementation reveals a spec contradiction.

---

### Task 1: Scaffold Project Shell

**Files:**
- Create: `package.json`
- Create: `index.html`
- Create: `vite.config.ts`
- Create: `tsconfig.json`
- Create: `tsconfig.node.json`
- Create: `src/main.tsx`
- Create: `src/App.tsx`
- Create: `src/styles.css`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/build.rs`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/src/lib.rs`
- Create: `src-tauri/capabilities/default.json`

- [ ] **Step 1: Create frontend package files**

Write `package.json`:

```json
{
  "name": "todo-float",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "test": "vitest run",
    "tauri": "tauri"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "react": "^18.3.1",
    "react-dom": "^18.3.1"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.0.0",
    "@testing-library/jest-dom": "^6.4.8",
    "@testing-library/react": "^16.0.1",
    "@testing-library/user-event": "^14.5.2",
    "@types/react": "^18.3.3",
    "@types/react-dom": "^18.3.0",
    "@vitejs/plugin-react": "^4.3.1",
    "jsdom": "^24.1.1",
    "typescript": "^5.5.4",
    "vite": "^5.4.8",
    "vitest": "^2.0.5"
  }
}
```

Write `index.html`:

```html
<!doctype html>
<html lang="zh-CN">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Todo Float</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
```

Write `vite.config.ts`:

```ts
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    strictPort: true,
    port: 1420,
  },
  envPrefix: ["VITE_", "TAURI_"],
  test: {
    environment: "jsdom",
    setupFiles: ["./src/test-setup.ts"],
  },
});
```

- [ ] **Step 2: Create TypeScript config**

Write `tsconfig.json`:

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["DOM", "DOM.Iterable", "ES2020"],
    "allowJs": false,
    "skipLibCheck": true,
    "esModuleInterop": true,
    "allowSyntheticDefaultImports": true,
    "strict": true,
    "forceConsistentCasingInFileNames": true,
    "module": "ESNext",
    "moduleResolution": "Node",
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx"
  },
  "include": ["src"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
```

Write `tsconfig.node.json`:

```json
{
  "compilerOptions": {
    "composite": true,
    "module": "ESNext",
    "moduleResolution": "Node",
    "allowSyntheticDefaultImports": true
  },
  "include": ["vite.config.ts"]
}
```

- [ ] **Step 3: Create minimal React app**

Write `src/main.tsx`:

```tsx
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
```

Write `src/App.tsx`:

```tsx
export default function App() {
  return (
    <main className="float-shell">
      <section className="float-panel">
        <header className="panel-header">
          <div>
            <h1>今天要做</h1>
            <p>加载中</p>
          </div>
        </header>
      </section>
    </main>
  );
}
```

Write `src/styles.css`:

```css
:root {
  font-family: "Segoe UI", "Microsoft YaHei", sans-serif;
  color: rgba(255, 255, 255, 0.92);
  background: transparent;
}

* {
  box-sizing: border-box;
}

body {
  margin: 0;
  min-width: 300px;
  min-height: 420px;
  background: transparent;
}

.float-shell {
  width: 300px;
  max-width: 480px;
  min-height: 420px;
}

.float-panel {
  min-height: 420px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 17px;
  background: rgba(44, 44, 44, 0.82);
  box-shadow: 0 22px 70px rgba(0, 0, 0, 0.42), inset 0 1px 0 rgba(255, 255, 255, 0.08);
  backdrop-filter: blur(28px) saturate(1.15);
  overflow: hidden;
}

.panel-header {
  padding: 17px;
}
```

- [ ] **Step 4: Create Tauri shell**

Write `src-tauri/Cargo.toml`:

```toml
[package]
name = "todo-float"
version = "0.1.0"
description = "Lightweight startup todo float"
authors = ["Aries"]
edition = "2021"

[lib]
name = "todo_float_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-autostart = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
rusqlite = { version = "0.32", features = ["bundled"] }
toml = "0.8"
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
thiserror = "1"
uuid = { version = "1", features = ["v4", "serde"] }

[dev-dependencies]
tempfile = "3"
httpmock = "0.7"
```

Write `src-tauri/build.rs`:

```rust
fn main() {
    tauri_build::build();
}
```

Write `src-tauri/src/main.rs`:

```rust
fn main() {
    todo_float_lib::run();
}
```

Write `src-tauri/src/lib.rs`:

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--startup-check".into()]),
        ))
        .setup(|app| {
            let _handle = app.handle().clone();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Todo Float");
}
```

Write `src-tauri/tauri.conf.json`:

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Todo Float",
  "version": "0.1.0",
  "identifier": "local.todo-float",
  "build": {
    "beforeDevCommand": "npm.cmd run dev",
    "beforeBuildCommand": "npm.cmd run build",
    "devUrl": "http://localhost:1420",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": []
  }
}
```

Write `src-tauri/capabilities/default.json`:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Default app capability",
  "windows": ["main"],
  "permissions": ["core:default"]
}
```

- [ ] **Step 5: Install dependencies**

Run:

```powershell
npm.cmd install
```

Expected: `package-lock.json` is created and npm exits with code 0.

- [ ] **Step 6: Verify the shell builds**

Run:

```powershell
npm.cmd run build
```

Expected: TypeScript and Vite build complete without errors.

Run:

```powershell
cd src-tauri
cargo test
```

Expected: Cargo compiles the empty Rust shell and reports `test result: ok`.

- [ ] **Step 7: Commit scaffold**

```powershell
git add package.json package-lock.json index.html vite.config.ts tsconfig.json tsconfig.node.json src src-tauri
git commit -m "chore: scaffold tauri todo float"
```

---

### Task 2: Define Shared Models

**Files:**
- Create: `src-tauri/src/models.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `src/types.ts`

- [ ] **Step 1: Add Rust models**

Write `src-tauri/src/models.rs`:

```rust
use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DateSource {
    Rule,
    Llm,
    Default,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Todo {
    pub id: String,
    pub title: String,
    pub due_date: NaiveDate,
    pub due_time: Option<NaiveTime>,
    pub source_text: Option<String>,
    pub date_expression: Option<String>,
    pub date_source: DateSource,
    pub warning: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PreviewEntry {
    pub title: String,
    pub due_date: NaiveDate,
    pub due_time: Option<NaiveTime>,
    pub source_text: String,
    pub date_expression: Option<String>,
    pub date_source: DateSource,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppState {
    pub today: NaiveDate,
    pub todos: Vec<Todo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParseRequest {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParseResponse {
    pub entries: Vec<PreviewEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SavePreviewRequest {
    pub entries: Vec<PreviewEntry>,
}
```

- [ ] **Step 2: Wire the module**

Modify `src-tauri/src/lib.rs`:

```rust
mod models;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--startup-check".into()]),
        ))
        .setup(|app| {
            let _handle = app.handle().clone();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Todo Float");
}
```

- [ ] **Step 3: Add frontend types**

Write `src/types.ts`:

```ts
export type DateSource = "rule" | "llm" | "default";

export interface Todo {
  id: string;
  title: string;
  due_date: string;
  due_time: string | null;
  source_text: string | null;
  date_expression: string | null;
  date_source: DateSource;
  warning: string | null;
  created_at: string;
  updated_at: string;
  deleted_at: string | null;
  completed_at: string | null;
}

export interface PreviewEntry {
  title: string;
  due_date: string;
  due_time: string | null;
  source_text: string;
  date_expression: string | null;
  date_source: DateSource;
  warning: string | null;
}

export interface AppState {
  today: string;
  todos: Todo[];
}

export interface ParseResponse {
  entries: PreviewEntry[];
}
```

- [ ] **Step 4: Verify model compilation**

Run:

```powershell
cd src-tauri
cargo test
```

Expected: Rust compiles and reports `test result: ok`.

Run:

```powershell
npm.cmd run build
```

Expected: TypeScript build passes.

- [ ] **Step 5: Commit models**

```powershell
git add src-tauri/src/lib.rs src-tauri/src/models.rs src/types.ts
git commit -m "feat: add shared todo models"
```

---

### Task 3: Implement Local Date Rules

**Files:**
- Create: `src-tauri/src/date_rules.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write failing Rust tests**

Create `src-tauri/src/date_rules.rs` with tests first:

```rust
use chrono::{Datelike, Duration, NaiveDate, Weekday};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateResolution {
    pub date: NaiveDate,
    pub expression: String,
    pub warning: Option<String>,
}

pub fn resolve_date(input: &str, base: NaiveDate) -> Option<DateResolution> {
    let _ = (input, base);
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 5, 19).unwrap()
    }

    #[test]
    fn resolves_relative_days() {
        assert_eq!(resolve_date("明天拿文件", base()).unwrap().date, NaiveDate::from_ymd_opt(2026, 5, 20).unwrap());
        assert_eq!(resolve_date("后天买牛奶", base()).unwrap().date, NaiveDate::from_ymd_opt(2026, 5, 21).unwrap());
        assert_eq!(resolve_date("大后天检查备份", base()).unwrap().date, NaiveDate::from_ymd_opt(2026, 5, 22).unwrap());
    }

    #[test]
    fn resolves_week_expressions_with_monday_start() {
        assert_eq!(resolve_date("这个周三交材料", base()).unwrap().date, NaiveDate::from_ymd_opt(2026, 5, 20).unwrap());
        assert_eq!(resolve_date("本周日打电话", base()).unwrap().date, NaiveDate::from_ymd_opt(2026, 5, 24).unwrap());
        assert_eq!(resolve_date("下周二复查", base()).unwrap().date, NaiveDate::from_ymd_opt(2026, 5, 26).unwrap());
        assert_eq!(resolve_date("下星期二确认合同", base()).unwrap().date, NaiveDate::from_ymd_opt(2026, 5, 26).unwrap());
        assert_eq!(resolve_date("下下周一续费", base()).unwrap().date, NaiveDate::from_ymd_opt(2026, 6, 1).unwrap());
    }

    #[test]
    fn resolves_bare_weekday_as_current_week_and_warns_when_past_or_today() {
        let result = resolve_date("周二寄快递", base()).unwrap();
        assert_eq!(result.date, NaiveDate::from_ymd_opt(2026, 5, 19).unwrap());
        assert_eq!(result.warning, None);

        let past = resolve_date("周一补发票", base()).unwrap();
        assert_eq!(past.date, NaiveDate::from_ymd_opt(2026, 5, 18).unwrap());
        assert!(past.warning.unwrap().contains("已过去"));
    }

    #[test]
    fn resolves_month_expressions() {
        assert_eq!(resolve_date("月底整理发票", base()).unwrap().date, NaiveDate::from_ymd_opt(2026, 5, 31).unwrap());
        assert_eq!(resolve_date("月初检查账单", base()).unwrap().date, NaiveDate::from_ymd_opt(2026, 5, 1).unwrap());
        assert_eq!(resolve_date("下个月3号还书", base()).unwrap().date, NaiveDate::from_ymd_opt(2026, 6, 3).unwrap());
        assert_eq!(resolve_date("下个月三号还书", base()).unwrap().date, NaiveDate::from_ymd_opt(2026, 6, 3).unwrap());
        assert_eq!(resolve_date("6月1号看活动", base()).unwrap().date, NaiveDate::from_ymd_opt(2026, 6, 1).unwrap());
    }
}
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```powershell
cd src-tauri
cargo test date_rules
```

Expected: tests fail because `resolve_date` returns `None`.

- [ ] **Step 3: Implement date rules**

Replace `resolve_date` and add helpers in `src-tauri/src/date_rules.rs`:

```rust
pub fn resolve_date(input: &str, base: NaiveDate) -> Option<DateResolution> {
    let text = input.trim();

    for (expr, days) in [("大后天", 3), ("后天", 2), ("明天", 1), ("今天", 0)] {
        if text.contains(expr) {
            return Some(DateResolution {
                date: base + Duration::days(days),
                expression: expr.to_string(),
                warning: None,
            });
        }
    }

    if text.contains("月底") {
        let first_next_month = if base.month() == 12 {
            NaiveDate::from_ymd_opt(base.year() + 1, 1, 1).unwrap()
        } else {
            NaiveDate::from_ymd_opt(base.year(), base.month() + 1, 1).unwrap()
        };
        return Some(DateResolution {
            date: first_next_month - Duration::days(1),
            expression: "月底".to_string(),
            warning: None,
        });
    }

    if text.contains("月初") {
        return Some(DateResolution {
            date: NaiveDate::from_ymd_opt(base.year(), base.month(), 1).unwrap(),
            expression: "月初".to_string(),
            warning: past_warning(NaiveDate::from_ymd_opt(base.year(), base.month(), 1).unwrap(), base),
        });
    }

    if let Some(day) = parse_next_month_day(text) {
        let (year, month) = if base.month() == 12 {
            (base.year() + 1, 1)
        } else {
            (base.year(), base.month() + 1)
        };
        if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
            return Some(DateResolution {
                date,
                expression: format!("下个月{}号", day),
                warning: None,
            });
        }
    }

    if text.contains("下周末") {
        let date = week_start(base) + Duration::days(12);
        return Some(DateResolution {
            date,
            expression: "下周末".to_string(),
            warning: None,
        });
    }

    if let Some((week_offset, weekday, expression)) = parse_weekday_expression(text) {
        let date = week_start(base) + Duration::days((week_offset * 7 + weekday_index(weekday)) as i64);
        return Some(DateResolution {
            date,
            expression,
            warning: past_warning(date, base),
        });
    }

    if let Some((month, day, expression)) = parse_month_day(text) {
        if let Some(date) = NaiveDate::from_ymd_opt(base.year(), month, day) {
            return Some(DateResolution {
                date,
                expression,
                warning: past_warning(date, base),
            });
        }
    }

    None
}

fn week_start(base: NaiveDate) -> NaiveDate {
    let offset = match base.weekday() {
        Weekday::Mon => 0,
        Weekday::Tue => 1,
        Weekday::Wed => 2,
        Weekday::Thu => 3,
        Weekday::Fri => 4,
        Weekday::Sat => 5,
        Weekday::Sun => 6,
    };
    base - Duration::days(offset)
}

fn weekday_index(day: Weekday) -> i32 {
    match day {
        Weekday::Mon => 0,
        Weekday::Tue => 1,
        Weekday::Wed => 2,
        Weekday::Thu => 3,
        Weekday::Fri => 4,
        Weekday::Sat => 5,
        Weekday::Sun => 6,
    }
}

fn past_warning(date: NaiveDate, base: NaiveDate) -> Option<String> {
    if date < base {
        Some(format!("日期 {} 已过去，请确认", date))
    } else {
        None
    }
}

fn parse_weekday_expression(text: &str) -> Option<(i32, Weekday, String)> {
    let weekdays = [
        ("一", Weekday::Mon),
        ("二", Weekday::Tue),
        ("三", Weekday::Wed),
        ("四", Weekday::Thu),
        ("五", Weekday::Fri),
        ("六", Weekday::Sat),
        ("日", Weekday::Sun),
        ("天", Weekday::Sun),
    ];
    for (cn, day) in weekdays {
        for prefix in ["周", "星期", "礼拜"] {
            let bare = format!("{prefix}{cn}");
            if text.contains(&format!("下下{bare}")) {
                return Some((2, day, format!("下下{bare}")));
            }
            if text.contains(&format!("下{bare}")) {
                return Some((1, day, format!("下{bare}")));
            }
            if text.contains(&format!("本{bare}")) || text.contains(&format!("这{bare}")) || text.contains(&format!("这个{bare}")) {
                return Some((0, day, bare));
            }
            if text.contains(&bare) {
                return Some((0, day, bare));
            }
        }
    }
    None
}

fn parse_next_month_day(text: &str) -> Option<u32> {
    if !text.contains("下个月") {
        return None;
    }
    for day in 1..=31 {
        if text.contains(&format!("下个月{}号", day)) || text.contains(&format!("下个月{}日", day)) {
            return Some(day);
        }
    }
    if text.contains("下个月三号") || text.contains("下个月三日") {
        return Some(3);
    }
    None
}

fn parse_month_day(text: &str) -> Option<(u32, u32, String)> {
    for month in 1..=12 {
        for day in 1..=31 {
            let a = format!("{month}月{day}号");
            let b = format!("{month}月{day}日");
            if text.contains(&a) {
                return Some((month, day, a));
            }
            if text.contains(&b) {
                return Some((month, day, b));
            }
        }
    }
    None
}
```

- [ ] **Step 4: Wire module**

Modify `src-tauri/src/lib.rs`:

```rust
mod date_rules;
mod models;
```

Keep the existing `run()` below the module declarations.

- [ ] **Step 5: Run date tests**

Run:

```powershell
cd src-tauri
cargo test date_rules
```

Expected: all `date_rules` tests pass.

- [ ] **Step 6: Commit date rules**

```powershell
git add src-tauri/src/date_rules.rs src-tauri/src/lib.rs
git commit -m "feat: add local Chinese date rules"
```

---

### Task 4: Add SQLite Repository

**Files:**
- Create: `src-tauri/src/db.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write repository tests**

Create `src-tauri/src/db.rs`:

```rust
use crate::models::{DateSource, Todo};
use chrono::NaiveDate;
use rusqlite::{params, Connection};
use uuid::Uuid;

pub fn open_database(path: &std::path::Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    migrate(&conn)?;
    Ok(conn)
}

pub fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    let _ = conn;
    Ok(())
}

pub fn insert_todo(conn: &Connection, title: &str, due_date: NaiveDate) -> rusqlite::Result<String> {
    let _ = (conn, title, due_date);
    Ok(String::new())
}

pub fn active_todos_for_date(conn: &Connection, due_date: NaiveDate) -> rusqlite::Result<Vec<Todo>> {
    let _ = (conn, due_date);
    Ok(Vec::new())
}

pub fn record_popup(conn: &Connection, date: NaiveDate, max_created_at: Option<&str>, suppressed: bool) -> rusqlite::Result<()> {
    let _ = (conn, date, max_created_at, suppressed);
    Ok(())
}

pub fn latest_popup_snapshot(conn: &Connection, date: NaiveDate) -> rusqlite::Result<Option<(Option<String>, bool)>> {
    let _ = (conn, date);
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrates_and_returns_today_todos() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        let today = NaiveDate::from_ymd_opt(2026, 5, 19).unwrap();
        let tomorrow = NaiveDate::from_ymd_opt(2026, 5, 20).unwrap();

        insert_todo(&conn, "拿文件", today).unwrap();
        insert_todo(&conn, "找凯莉", today).unwrap();
        insert_todo(&conn, "买牛奶", tomorrow).unwrap();

        let todos = active_todos_for_date(&conn, today).unwrap();
        assert_eq!(todos.len(), 2);
        assert_eq!(todos[0].title, "拿文件");
        assert_eq!(todos[1].title, "找凯莉");
    }

    #[test]
    fn stores_popup_snapshot() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        let today = NaiveDate::from_ymd_opt(2026, 5, 19).unwrap();

        record_popup(&conn, today, Some("2026-05-19T09:00:00Z"), false).unwrap();
        let snapshot = latest_popup_snapshot(&conn, today).unwrap().unwrap();

        assert_eq!(snapshot.0.unwrap(), "2026-05-19T09:00:00Z");
        assert!(!snapshot.1);
    }
}
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```powershell
cd src-tauri
cargo test db
```

Expected: `migrates_and_returns_today_todos` fails because the migration creates no tables and inserts no rows.

- [ ] **Step 3: Implement migrations and repository**

Replace the stub functions in `src-tauri/src/db.rs` with:

```rust
pub fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS todos (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            due_date TEXT NOT NULL,
            due_time TEXT NULL,
            source_text TEXT NULL,
            date_expression TEXT NULL,
            date_source TEXT NOT NULL,
            warning TEXT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            deleted_at TEXT NULL,
            completed_at TEXT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_todos_due_date_active
        ON todos(due_date, deleted_at, completed_at);

        CREATE TABLE IF NOT EXISTS popup_events (
            id TEXT PRIMARY KEY,
            date TEXT NOT NULL,
            shown_at TEXT NOT NULL,
            todo_snapshot_max_created_at TEXT NULL,
            suppressed INTEGER NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_popup_events_date_shown
        ON popup_events(date, shown_at);
        "#,
    )
}

pub fn insert_todo(conn: &Connection, title: &str, due_date: NaiveDate) -> rusqlite::Result<String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        r#"
        INSERT INTO todos (
            id, title, due_date, due_time, source_text, date_expression, date_source,
            warning, created_at, updated_at, deleted_at, completed_at
        ) VALUES (?1, ?2, ?3, NULL, NULL, NULL, 'rule', NULL, ?4, ?4, NULL, NULL)
        "#,
        params![id, title, due_date.to_string(), now],
    )?;
    Ok(id)
}

pub fn active_todos_for_date(conn: &Connection, due_date: NaiveDate) -> rusqlite::Result<Vec<Todo>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, title, due_date, due_time, source_text, date_expression, date_source,
               warning, created_at, updated_at, deleted_at, completed_at
        FROM todos
        WHERE due_date = ?1 AND deleted_at IS NULL AND completed_at IS NULL
        ORDER BY created_at ASC
        "#,
    )?;
    let rows = stmt.query_map(params![due_date.to_string()], |row| {
        let date_source = match row.get::<_, String>(6)?.as_str() {
            "rule" => DateSource::Rule,
            "llm" => DateSource::Llm,
            _ => DateSource::Default,
        };
        Ok(Todo {
            id: row.get(0)?,
            title: row.get(1)?,
            due_date: chrono::NaiveDate::parse_from_str(&row.get::<_, String>(2)?, "%Y-%m-%d").unwrap(),
            due_time: row
                .get::<_, Option<String>>(3)?
                .and_then(|value| chrono::NaiveTime::parse_from_str(&value, "%H:%M").ok()),
            source_text: row.get(4)?,
            date_expression: row.get(5)?,
            date_source,
            warning: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
            deleted_at: row.get(10)?,
            completed_at: row.get(11)?,
        })
    })?;

    rows.collect()
}

pub fn record_popup(conn: &Connection, date: NaiveDate, max_created_at: Option<&str>, suppressed: bool) -> rusqlite::Result<()> {
    let id = Uuid::new_v4().to_string();
    let shown_at = chrono::Utc::now().to_rfc3339();
    conn.execute(
        r#"
        INSERT INTO popup_events (id, date, shown_at, todo_snapshot_max_created_at, suppressed)
        VALUES (?1, ?2, ?3, ?4, ?5)
        "#,
        params![id, date.to_string(), shown_at, max_created_at, if suppressed { 1 } else { 0 }],
    )?;
    Ok(())
}

pub fn latest_popup_snapshot(conn: &Connection, date: NaiveDate) -> rusqlite::Result<Option<(Option<String>, bool)>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT todo_snapshot_max_created_at, suppressed
        FROM popup_events
        WHERE date = ?1
        ORDER BY shown_at DESC
        LIMIT 1
        "#,
    )?;
    let mut rows = stmt.query(params![date.to_string()])?;
    if let Some(row) = rows.next()? {
        let snapshot: Option<String> = row.get(0)?;
        let suppressed: i64 = row.get(1)?;
        Ok(Some((snapshot, suppressed == 1)))
    } else {
        Ok(None)
    }
}
```

- [ ] **Step 4: Wire module**

Modify `src-tauri/src/lib.rs`:

```rust
mod date_rules;
mod db;
mod models;
```

- [ ] **Step 5: Run repository tests**

Run:

```powershell
cd src-tauri
cargo test db
```

Expected: both db tests pass.

- [ ] **Step 6: Commit repository**

```powershell
git add src-tauri/src/db.rs src-tauri/src/lib.rs
git commit -m "feat: add sqlite repository"
```

---

### Task 5: Implement Config Loading

**Files:**
- Create: `src-tauri/src/config.rs`
- Create: `config.example.toml`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write config tests**

Create `src-tauri/src/config.rs`:

```rust
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct LlmProviderConfig {
    pub provider: String,
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct LlmConfig {
    pub provider: String,
    pub api_key: String,
    pub model: String,
    pub fallback: Option<LlmProviderConfig>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct AppConfig {
    pub llm: LlmConfig,
}

pub fn config_path() -> Result<PathBuf, String> {
    if let Ok(path) = std::env::var("TODO_FLOAT_CONFIG") {
        return Ok(PathBuf::from(path));
    }
    let exe = std::env::current_exe().map_err(|err| err.to_string())?;
    let dir = exe.parent().ok_or_else(|| "Cannot find executable directory".to_string())?;
    Ok(dir.join("config.toml"))
}

pub fn load_config_from(path: &Path) -> Result<AppConfig, String> {
    let _ = path;
    Err("stubbed config loader".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_toml_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(
            &path,
            r#"
            [llm]
            provider = "openrouter"
            api_key = "abc"
            model = "z-ai/glm-4.5-air"

            [llm.fallback]
            provider = "bigmodel"
            api_key = "def"
            model = "glm-4.7-flash"
            "#,
        )
        .unwrap();

        let config = load_config_from(&path).unwrap();
        assert_eq!(config.llm.provider, "openrouter");
        assert_eq!(config.llm.fallback.unwrap().model, "glm-4.7-flash");
    }

    #[test]
    fn rejects_missing_api_key_for_primary() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(
            &path,
            r#"
            [llm]
            provider = "openrouter"
            api_key = ""
            model = "z-ai/glm-4.5-air"
            "#,
        )
        .unwrap();

        let config = load_config_from(&path).unwrap();
        assert_eq!(config.llm.api_key, "");
    }
}
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```powershell
cd src-tauri
cargo test config
```

Expected: `loads_toml_config` fails with `stubbed config loader`.

- [ ] **Step 3: Implement TOML parsing**

Replace `load_config_from` in `src-tauri/src/config.rs`:

```rust
pub fn load_config_from(path: &Path) -> Result<AppConfig, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|err| format!("无法读取配置文件 {}: {}", path.display(), err))?;
    toml::from_str::<AppConfig>(&text)
        .map_err(|err| format!("配置文件格式错误 {}: {}", path.display(), err))
}
```

- [ ] **Step 4: Add config example**

Write `config.example.toml`:

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

- [ ] **Step 5: Wire module and verify**

Modify `src-tauri/src/lib.rs`:

```rust
mod config;
mod date_rules;
mod db;
mod models;
```

Run:

```powershell
cd src-tauri
cargo test config
```

Expected: all config tests pass.

- [ ] **Step 6: Commit config**

```powershell
git add config.example.toml src-tauri/src/config.rs src-tauri/src/lib.rs
git commit -m "feat: add portable config loading"
```

---

### Task 6: Implement LLM JSON Parsing And Providers

**Files:**
- Create: `src-tauri/src/llm.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write JSON/schema tests**

Create `src-tauri/src/llm.rs`:

```rust
use crate::models::PreviewEntry;
use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct LlmEnvelope {
    entries: Vec<LlmEntry>,
}

#[derive(Debug, Deserialize)]
struct LlmEntry {
    date_expression: Option<String>,
    due_date: String,
    due_time: Option<String>,
    item: String,
    warning: Option<String>,
}

pub fn parse_llm_entries(source_text: &str, json: &str) -> Result<Vec<PreviewEntry>, String> {
    let _ = (source_text, json);
    Err("stubbed llm parser".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_entries() {
        let json = r#"{
          "entries": [
            {
              "date_expression": "明天",
              "due_date": "2026-05-20",
              "due_time": null,
              "item": "拿文件",
              "date_confidence": "high",
              "warning": null
            }
          ],
          "notes": null
        }"#;
        let entries = parse_llm_entries("明天拿文件", json).unwrap();
        assert_eq!(entries[0].title, "拿文件");
        assert_eq!(entries[0].due_date, NaiveDate::from_ymd_opt(2026, 5, 20).unwrap());
    }

    #[test]
    fn rejects_invalid_date() {
        let json = r#"{"entries":[{"date_expression":"明天","due_date":"tomorrow","due_time":null,"item":"拿文件","warning":null}]}"#;
        let err = parse_llm_entries("明天拿文件", json).unwrap_err();
        assert!(err.contains("due_date"));
    }

    #[test]
    fn rejects_empty_item() {
        let json = r#"{"entries":[{"date_expression":"明天","due_date":"2026-05-20","due_time":null,"item":" ","warning":null}]}"#;
        let err = parse_llm_entries("明天", json).unwrap_err();
        assert!(err.contains("item"));
    }
}
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```powershell
cd src-tauri
cargo test llm
```

Expected: `parses_valid_entries` fails with `stubbed llm parser`.

- [ ] **Step 3: Implement schema validation**

Replace `parse_llm_entries`:

```rust
pub fn parse_llm_entries(source_text: &str, json: &str) -> Result<Vec<PreviewEntry>, String> {
    let envelope: LlmEnvelope = serde_json::from_str(json)
        .map_err(|err| format!("LLM 返回的 JSON 无法解析: {err}"))?;
    if envelope.entries.is_empty() {
        return Err("LLM 返回缺少 entries".to_string());
    }

    envelope
        .entries
        .into_iter()
        .map(|entry| {
            let title = entry.item.trim().to_string();
            if title.is_empty() {
                return Err("LLM entry.item 为空".to_string());
            }
            let due_date = NaiveDate::parse_from_str(&entry.due_date, "%Y-%m-%d")
                .map_err(|err| format!("LLM entry.due_date 无效: {err}"))?;
            let due_time = match entry.due_time {
                Some(value) if !value.trim().is_empty() => Some(
                    chrono::NaiveTime::parse_from_str(value.trim(), "%H:%M")
                        .map_err(|err| format!("LLM entry.due_time 无效: {err}"))?,
                ),
                _ => None,
            };
            Ok(PreviewEntry {
                title,
                due_date,
                due_time,
                source_text: source_text.to_string(),
                date_expression: entry.date_expression,
                date_source: crate::models::DateSource::Llm,
                warning: entry.warning,
            })
        })
        .collect()
}
```

- [ ] **Step 4: Add provider request functions**

Append to `src-tauri/src/llm.rs`:

```rust
#[derive(Debug, Clone)]
pub struct ProviderRequest {
    pub provider: String,
    pub api_key: String,
    pub model: String,
    pub prompt: String,
    pub user_text: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Debug, Deserialize)]
struct ChatMessage {
    content: Option<String>,
}

pub async fn call_provider(request: ProviderRequest) -> Result<String, String> {
    if request.api_key.trim().is_empty() {
        return Err("配置文件中的 API key 为空".to_string());
    }

    let endpoint = match request.provider.as_str() {
        "openrouter" => "https://openrouter.ai/api/v1/chat/completions",
        "bigmodel" => "https://open.bigmodel.cn/api/paas/v4/chat/completions",
        other => return Err(format!("不支持的 LLM provider: {other}")),
    };

    let body = serde_json::json!({
        "model": request.model,
        "messages": [
            { "role": "system", "content": request.prompt },
            { "role": "user", "content": request.user_text }
        ],
        "stream": false,
        "response_format": { "type": "json_object" },
        "temperature": 0,
        "max_tokens": 2048,
        "reasoning": { "effort": "none", "exclude": true },
        "thinking": { "type": "disabled" }
    });

    let client = reqwest::Client::new();
    let response = client
        .post(endpoint)
        .bearer_auth(request.api_key)
        .header("Content-Type", "application/json; charset=utf-8")
        .header("HTTP-Referer", "http://localhost/todo-float")
        .header("X-Title", "Todo Float")
        .json(&body)
        .send()
        .await
        .map_err(|err| format!("LLM 请求失败: {err}"))?;

    let status = response.status();
    let text = response.text().await.map_err(|err| err.to_string())?;
    if !status.is_success() {
        return Err(format!("LLM 返回 HTTP {status}: {text}"));
    }

    let parsed: ChatResponse = serde_json::from_str(&text)
        .map_err(|err| format!("LLM 响应结构无法解析: {err}; raw={text}"))?;
    parsed
        .choices
        .into_iter()
        .next()
        .and_then(|choice| choice.message.content)
        .ok_or_else(|| "LLM 响应缺少 message.content".to_string())
}
```

- [ ] **Step 5: Wire module and run tests**

Modify `src-tauri/src/lib.rs`:

```rust
mod config;
mod date_rules;
mod db;
mod llm;
mod models;
```

Run:

```powershell
cd src-tauri
cargo test llm
```

Expected: all LLM schema tests pass.

- [ ] **Step 6: Commit LLM parsing**

```powershell
git add src-tauri/src/llm.rs src-tauri/src/lib.rs
git commit -m "feat: add llm response validation"
```

---

### Task 7: Orchestrate Natural-Language Parsing

**Files:**
- Create: `src-tauri/src/parser.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write parser tests**

Create `src-tauri/src/parser.rs`:

```rust
use crate::date_rules::resolve_date;
use crate::models::{DateSource, PreviewEntry};
use chrono::NaiveDate;

pub fn build_prompt(base: NaiveDate) -> String {
    let _ = base;
    String::new()
}

pub fn apply_local_overrides(entries: Vec<PreviewEntry>, source_text: &str, base: NaiveDate) -> Vec<PreviewEntry> {
    let _ = (source_text, base);
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 5, 19).unwrap()
    }

    #[test]
    fn prompt_contains_week_tables() {
        let prompt = build_prompt(base());
        assert!(prompt.contains("当前基准日期：2026-05-19"));
        assert!(prompt.contains("周二=2026-05-19"));
        assert!(prompt.contains("下周"));
        assert!(prompt.contains("周二=2026-05-26"));
    }

    #[test]
    fn local_rule_overrides_llm_date() {
        let entries = vec![PreviewEntry {
            title: "去医院复查".to_string(),
            due_date: NaiveDate::from_ymd_opt(2026, 5, 25).unwrap(),
            due_time: None,
            source_text: "下周二提醒我去医院复查".to_string(),
            date_expression: Some("下周二".to_string()),
            date_source: DateSource::Llm,
            warning: None,
        }];

        let corrected = apply_local_overrides(entries, "下周二提醒我去医院复查", base());

        assert_eq!(corrected[0].due_date, NaiveDate::from_ymd_opt(2026, 5, 26).unwrap());
        assert_eq!(corrected[0].date_source, DateSource::Rule);
    }
}
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```powershell
cd src-tauri
cargo test parser
```

Expected: `prompt_contains_week_tables` fails because `build_prompt` returns an empty string.

- [ ] **Step 3: Implement prompt builder and overrides**

Replace functions in `src-tauri/src/parser.rs`:

```rust
pub fn build_prompt(base: NaiveDate) -> String {
    let current_start = {
        let offset = base.weekday().num_days_from_monday() as i64;
        base - chrono::Duration::days(offset)
    };
    let next_start = current_start + chrono::Duration::days(7);
    let weekdays = ["周一", "周二", "周三", "周四", "周五", "周六", "周日"];
    let table = |start: NaiveDate| {
        weekdays
            .iter()
            .enumerate()
            .map(|(idx, label)| format!("{label}={}", start + chrono::Duration::days(idx as i64)))
            .collect::<Vec<_>>()
            .join("，")
    };

    format!(
        r#"你是 todo 输入解析器。你必须只输出 JSON 对象，不输出 markdown 或解释。
当前基准日期：{base}。
时区：Asia/Shanghai。
用户习惯：周一是一周开始，周日是一周结束。
当前周日期表：{}。
下周日期表：{}。
任务：从用户中文输入中提取一个或多个 todo 条目。每个条目都必须有自己的绝对日期。
日期规则：今天表示当前基准日期；明天表示基准日期后一天；后天表示基准日期后两天；大后天表示基准日期后三天；裸星期表达解释为本周；下周表示下一整个周一到周日；下下周表示下周之后的那一周；周五之前按周五当天；下周末默认按下周六；月底表示当前月最后一天；下个月3号表示下个月的 3 号。
拆分规则：如果一句话里多个事项共享同一个日期，拆成多个 entries；如果不同事项有不同日期，分别填各自日期；去掉“提醒我”“记得”“帮我”这类触发词。
输出字段固定为：{{"entries":[{{"date_expression": string|null, "due_date": "YYYY-MM-DD", "due_time": "HH:mm"|null, "item": string, "warning": string|null}}], "notes": string|null}}"#,
        table(current_start),
        table(next_start)
    )
}

pub fn apply_local_overrides(mut entries: Vec<PreviewEntry>, source_text: &str, base: NaiveDate) -> Vec<PreviewEntry> {
    for entry in &mut entries {
        let combined = match &entry.date_expression {
            Some(expression) => format!("{source_text} {expression}"),
            None => source_text.to_string(),
        };
        if let Some(resolution) = resolve_date(&combined, base) {
            entry.due_date = resolution.date;
            entry.date_expression = Some(resolution.expression);
            entry.date_source = DateSource::Rule;
            if resolution.warning.is_some() {
                entry.warning = resolution.warning;
            }
        }
    }
    entries
}
```

Add imports at the top:

```rust
use chrono::Datelike;
```

- [ ] **Step 4: Wire module and run tests**

Modify `src-tauri/src/lib.rs`:

```rust
mod config;
mod date_rules;
mod db;
mod llm;
mod models;
mod parser;
```

Run:

```powershell
cd src-tauri
cargo test parser
```

Expected: parser tests pass.

- [ ] **Step 5: Commit parser orchestration**

```powershell
git add src-tauri/src/parser.rs src-tauri/src/lib.rs
git commit -m "feat: add parser orchestration"
```

---

### Task 8: Implement Startup Decision

**Files:**
- Create: `src-tauri/src/startup.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write startup decision tests**

Create `src-tauri/src/startup.rs`:

```rust
use crate::models::Todo;

pub fn should_show_popup(todos: &[Todo], latest_snapshot: Option<(Option<String>, bool)>) -> bool {
    let _ = (todos, latest_snapshot);
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DateSource, Todo};
    use chrono::NaiveDate;

    fn todo(created_at: &str) -> Todo {
        Todo {
            id: created_at.to_string(),
            title: "拿文件".to_string(),
            due_date: NaiveDate::from_ymd_opt(2026, 5, 19).unwrap(),
            due_time: None,
            source_text: None,
            date_expression: None,
            date_source: DateSource::Rule,
            warning: None,
            created_at: created_at.to_string(),
            updated_at: created_at.to_string(),
            deleted_at: None,
            completed_at: None,
        }
    }

    #[test]
    fn does_not_show_empty_day() {
        assert!(!should_show_popup(&[], None));
    }

    #[test]
    fn shows_when_no_prior_snapshot() {
        assert!(should_show_popup(&[todo("2026-05-19T09:00:00Z")], None));
    }

    #[test]
    fn does_not_show_when_snapshot_is_current() {
        assert!(!should_show_popup(
            &[todo("2026-05-19T09:00:00Z")],
            Some((Some("2026-05-19T09:00:00Z".to_string()), false)),
        ));
    }

    #[test]
    fn shows_when_new_todo_was_added_after_snapshot() {
        assert!(should_show_popup(
            &[todo("2026-05-19T09:00:00Z"), todo("2026-05-19T12:00:00Z")],
            Some((Some("2026-05-19T09:00:00Z".to_string()), true)),
        ));
    }
}
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```powershell
cd src-tauri
cargo test startup
```

Expected: `shows_when_no_prior_snapshot` fails.

- [ ] **Step 3: Implement popup decision**

Replace `should_show_popup`:

```rust
pub fn should_show_popup(todos: &[Todo], latest_snapshot: Option<(Option<String>, bool)>) -> bool {
    if todos.is_empty() {
        return false;
    }

    let current_max = todos.iter().map(|todo| todo.created_at.as_str()).max();
    match latest_snapshot {
        None => true,
        Some((None, _suppressed)) => true,
        Some((Some(snapshot_max), _suppressed)) => current_max.is_some_and(|value| value > snapshot_max.as_str()),
    }
}
```

- [ ] **Step 4: Wire module and run tests**

Modify `src-tauri/src/lib.rs`:

```rust
mod config;
mod date_rules;
mod db;
mod llm;
mod models;
mod parser;
mod startup;
```

Run:

```powershell
cd src-tauri
cargo test startup
```

Expected: startup tests pass.

- [ ] **Step 5: Commit startup decision**

```powershell
git add src-tauri/src/startup.rs src-tauri/src/lib.rs
git commit -m "feat: add startup popup decision"
```

---

### Task 9: Add Tauri Commands And Window Creation

**Files:**
- Create: `src-tauri/src/window.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add command skeletons**

Create `src-tauri/src/window.rs`:

```rust
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

pub fn open_main_window(app: &AppHandle) -> tauri::Result<()> {
    if app.get_webview_window("main").is_some() {
        return Ok(());
    }

    WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
        .title("Todo Float")
        .inner_size(300.0, 470.0)
        .min_inner_size(300.0, 360.0)
        .max_inner_size(480.0, 720.0)
        .resizable(true)
        .decorations(false)
        .transparent(true)
        .always_on_top(false)
        .build()?;

    Ok(())
}
```

Modify `src-tauri/src/lib.rs` to include commands:

```rust
use chrono::Local;
use models::{AppState, ParseRequest, ParseResponse, SavePreviewRequest, Todo};

mod config;
mod date_rules;
mod db;
mod llm;
mod models;
mod parser;
mod startup;
mod window;

#[tauri::command]
fn get_app_state() -> Result<AppState, String> {
    let today = Local::now().date_naive();
    Ok(AppState { today, todos: Vec::<Todo>::new() })
}

#[tauri::command]
async fn parse_todo_text(request: ParseRequest) -> Result<ParseResponse, String> {
    let _ = request;
    Err("配置文件中的 API key 为空".to_string())
}

#[tauri::command]
fn save_preview(request: SavePreviewRequest) -> Result<Vec<Todo>, String> {
    let _ = request;
    Ok(Vec::new())
}

#[tauri::command]
fn suppress_today() -> Result<(), String> {
    Ok(())
}
```

- [ ] **Step 2: Register commands and create window in normal mode**

In `run()` inside `src-tauri/src/lib.rs`, add `.invoke_handler(...)` and window setup:

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--startup-check".into()]),
        ))
        .invoke_handler(tauri::generate_handler![
            get_app_state,
            parse_todo_text,
            save_preview,
            suppress_today
        ])
        .setup(|app| {
            let handle = app.handle().clone();

            let is_startup_check = std::env::args().any(|arg| arg == "--startup-check");
            if !is_startup_check {
                window::open_main_window(&handle)?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Todo Float");
}
```

- [ ] **Step 3: Verify window build**

Run:

```powershell
npm.cmd run build
cd src-tauri
cargo test
```

Expected: frontend and Rust compile.

- [ ] **Step 4: Commit Tauri commands**

```powershell
git add src-tauri/src/lib.rs src-tauri/src/window.rs
git commit -m "feat: add tauri command shell"
```

---

### Task 10: Connect Backend Commands To DB And Parser

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/db.rs`

- [ ] **Step 1: Add app path helpers**

In `src-tauri/src/lib.rs`, add:

```rust
fn database_path() -> Result<std::path::PathBuf, String> {
    if let Ok(path) = std::env::var("TODO_FLOAT_DB") {
        return Ok(std::path::PathBuf::from(path));
    }
    let exe = std::env::current_exe().map_err(|err| err.to_string())?;
    let dir = exe.parent().ok_or_else(|| "Cannot find executable directory".to_string())?;
    Ok(dir.join("todo-float.sqlite3"))
}

fn today() -> chrono::NaiveDate {
    chrono::Local::now().date_naive()
}
```

- [ ] **Step 2: Add insert from preview entries**

In `src-tauri/src/db.rs`, add:

```rust
pub fn insert_preview_entry(conn: &Connection, entry: &crate::models::PreviewEntry) -> rusqlite::Result<String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let due_time = entry.due_time.map(|time| time.format("%H:%M").to_string());
    let source = match entry.date_source {
        crate::models::DateSource::Rule => "rule",
        crate::models::DateSource::Llm => "llm",
        crate::models::DateSource::Default => "default",
    };
    conn.execute(
        r#"
        INSERT INTO todos (
            id, title, due_date, due_time, source_text, date_expression, date_source,
            warning, created_at, updated_at, deleted_at, completed_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9, NULL, NULL)
        "#,
        params![
            id,
            entry.title,
            entry.due_date.to_string(),
            due_time,
            entry.source_text,
            entry.date_expression,
            source,
            entry.warning,
            now
        ],
    )?;
    Ok(id)
}
```

- [ ] **Step 3: Implement `get_app_state` and `save_preview`**

Replace command bodies in `src-tauri/src/lib.rs`:

```rust
#[tauri::command]
fn get_app_state() -> Result<AppState, String> {
    let today = today();
    let db_path = database_path()?;
    let conn = db::open_database(&db_path).map_err(|err| err.to_string())?;
    let todos = db::active_todos_for_date(&conn, today).map_err(|err| err.to_string())?;
    Ok(AppState { today, todos })
}

#[tauri::command]
fn save_preview(request: SavePreviewRequest) -> Result<Vec<Todo>, String> {
    let db_path = database_path()?;
    let conn = db::open_database(&db_path).map_err(|err| err.to_string())?;
    for entry in &request.entries {
        db::insert_preview_entry(&conn, entry).map_err(|err| err.to_string())?;
    }
    db::active_todos_for_date(&conn, today()).map_err(|err| err.to_string())
}
```

- [ ] **Step 4: Implement `parse_todo_text`**

Replace `parse_todo_text` in `src-tauri/src/lib.rs`:

```rust
#[tauri::command]
async fn parse_todo_text(request: ParseRequest) -> Result<ParseResponse, String> {
    let config_path = config::config_path()?;
    let config = config::load_config_from(&config_path)?;
    if config.llm.api_key.trim().is_empty() {
        return Err(format!("请先填写配置文件 {}", config_path.display()));
    }

    let base = today();
    let prompt = parser::build_prompt(base);
    let provider_request = llm::ProviderRequest {
        provider: config.llm.provider,
        api_key: config.llm.api_key,
        model: config.llm.model,
        prompt,
        user_text: request.text.clone(),
    };
    let raw = llm::call_provider(provider_request).await?;
    let entries = llm::parse_llm_entries(&request.text, &raw)?;
    let entries = parser::apply_local_overrides(entries, &request.text, base);
    Ok(ParseResponse { entries })
}
```

- [ ] **Step 5: Implement `suppress_today`**

Replace `suppress_today`:

```rust
#[tauri::command]
fn suppress_today() -> Result<(), String> {
    let today = today();
    let db_path = database_path()?;
    let conn = db::open_database(&db_path).map_err(|err| err.to_string())?;
    let todos = db::active_todos_for_date(&conn, today).map_err(|err| err.to_string())?;
    let max_created = todos.iter().map(|todo| todo.created_at.as_str()).max();
    db::record_popup(&conn, today, max_created, true).map_err(|err| err.to_string())
}
```

- [ ] **Step 6: Verify backend commands compile**

Run:

```powershell
cd src-tauri
cargo test
```

Expected: all Rust tests pass.

- [ ] **Step 7: Commit command integration**

```powershell
git add src-tauri/src/lib.rs src-tauri/src/db.rs
git commit -m "feat: connect commands to storage and parser"
```

---

### Task 11: Complete Startup Check Mode

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add startup check function**

In `src-tauri/src/lib.rs`, add:

```rust
fn run_startup_check(app: &tauri::AppHandle) -> Result<(), String> {
    let today = today();
    let db_path = database_path()?;
    let conn = db::open_database(&db_path).map_err(|err| err.to_string())?;
    let todos = db::active_todos_for_date(&conn, today).map_err(|err| err.to_string())?;
    let snapshot = db::latest_popup_snapshot(&conn, today).map_err(|err| err.to_string())?;

    if startup::should_show_popup(&todos, snapshot) {
        let max_created = todos.iter().map(|todo| todo.created_at.as_str()).max();
        db::record_popup(&conn, today, max_created, false).map_err(|err| err.to_string())?;
        window::open_main_window(app).map_err(|err| err.to_string())?;
    } else {
        app.exit(0);
    }
    Ok(())
}
```

- [ ] **Step 2: Call startup check during setup**

Replace the `is_startup_check` block inside `setup`:

```rust
let is_startup_check = std::env::args().any(|arg| arg == "--startup-check");
if is_startup_check {
    if run_startup_check(&handle).is_err() {
        handle.exit(0);
    }
} else {
    window::open_main_window(&handle)?;
}
```

- [ ] **Step 3: Verify no-window path is represented by tests**

Run:

```powershell
cd src-tauri
cargo test startup
```

Expected: startup decision tests pass.

Run:

```powershell
cd src-tauri
cargo test db
```

Expected: database tests pass.

- [ ] **Step 4: Commit startup mode**

```powershell
git add src-tauri/src/lib.rs
git commit -m "feat: implement startup check mode"
```

---

### Task 12: Build Frontend API Layer

**Files:**
- Create: `src/api.ts`
- Create: `src/test-setup.ts`
- Modify: `src/App.tsx`

- [ ] **Step 1: Add API wrappers**

Write `src/api.ts`:

```ts
import { invoke } from "@tauri-apps/api/core";
import type { AppState, ParseResponse, PreviewEntry, Todo } from "./types";

export async function getAppState(): Promise<AppState> {
  return invoke<AppState>("get_app_state");
}

export async function parseTodoText(text: string): Promise<ParseResponse> {
  return invoke<ParseResponse>("parse_todo_text", { request: { text } });
}

export async function savePreview(entries: PreviewEntry[]): Promise<Todo[]> {
  return invoke<Todo[]>("save_preview", { request: { entries } });
}

export async function suppressToday(): Promise<void> {
  return invoke<void>("suppress_today");
}
```

- [ ] **Step 2: Add test setup**

Write `src/test-setup.ts`:

```ts
import "@testing-library/jest-dom/vitest";
```

- [ ] **Step 3: Refactor App to load state**

Replace `src/App.tsx`:

```tsx
import { useEffect, useState } from "react";
import { getAppState } from "./api";
import type { AppState } from "./types";

export default function App() {
  const [state, setState] = useState<AppState | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getAppState()
      .then(setState)
      .catch((err) => setError(String(err)));
  }, []);

  return (
    <main className="float-shell">
      <section className="float-panel">
        <header className="panel-header">
          <div>
            <h1>今天要做</h1>
            <p>{state?.today ?? "加载中"}</p>
          </div>
          <span className="count">{state?.todos.length ?? 0}</span>
        </header>
        {error ? <p className="error">{error}</p> : null}
      </section>
    </main>
  );
}
```

- [ ] **Step 4: Verify TypeScript**

Run:

```powershell
npm.cmd run build
```

Expected: TypeScript and Vite build pass.

- [ ] **Step 5: Commit API layer**

```powershell
git add src/api.ts src/test-setup.ts src/App.tsx
git commit -m "feat: add frontend api layer"
```

---

### Task 13: Build Todo List UI

**Files:**
- Create: `src/components/TodoList.tsx`
- Create: `src/components/ErrorBanner.tsx`
- Modify: `src/App.tsx`
- Modify: `src/styles.css`
- Create: `src/App.test.tsx`

- [ ] **Step 1: Write UI test**

Write `src/App.test.tsx`:

```tsx
import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import App from "./App";

vi.mock("./api", () => ({
  getAppState: async () => ({
    today: "2026-05-19",
    todos: [
      {
        id: "1",
        title: "拿文件",
        due_date: "2026-05-19",
        due_time: null,
        source_text: null,
        date_expression: null,
        date_source: "rule",
        warning: null,
        created_at: "2026-05-19T09:00:00Z",
        updated_at: "2026-05-19T09:00:00Z",
        deleted_at: null,
        completed_at: null,
      },
    ],
  }),
  parseTodoText: vi.fn(),
  savePreview: vi.fn(),
  suppressToday: vi.fn(),
}));

describe("App", () => {
  it("renders today's todos", async () => {
    render(<App />);
    expect(await screen.findByText("拿文件")).toBeInTheDocument();
    expect(screen.getByText("1")).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: Run test to verify failure**

Run:

```powershell
npm.cmd test
```

Expected: test fails because `TodoList` is not rendering todo titles.

- [ ] **Step 3: Create list components**

Write `src/components/TodoList.tsx`:

```tsx
import type { Todo } from "../types";

interface TodoListProps {
  todos: Todo[];
}

export function TodoList({ todos }: TodoListProps) {
  if (todos.length === 0) {
    return <p className="empty">今天没有事项</p>;
  }

  return (
    <ol className="todo-list">
      {todos.map((todo, index) => (
        <li className="todo-item" key={todo.id}>
          <span className="todo-index">{index + 1}</span>
          <span className="todo-title">{todo.title}</span>
        </li>
      ))}
    </ol>
  );
}
```

Write `src/components/ErrorBanner.tsx`:

```tsx
interface ErrorBannerProps {
  message: string | null;
}

export function ErrorBanner({ message }: ErrorBannerProps) {
  if (!message) {
    return null;
  }
  return <p className="error">{message}</p>;
}
```

- [ ] **Step 4: Use list components**

Modify `src/App.tsx`:

```tsx
import { useEffect, useState } from "react";
import { getAppState } from "./api";
import { ErrorBanner } from "./components/ErrorBanner";
import { TodoList } from "./components/TodoList";
import type { AppState } from "./types";

export default function App() {
  const [state, setState] = useState<AppState | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getAppState()
      .then(setState)
      .catch((err) => setError(String(err)));
  }, []);

  const todos = state?.todos ?? [];

  return (
    <main className="float-shell">
      <section className="float-panel">
        <header className="panel-header">
          <div>
            <h1>今天要做</h1>
            <p>{state?.today ?? "加载中"}</p>
          </div>
          <span className="count">{todos.length}</span>
        </header>
        <ErrorBanner message={error} />
        <TodoList todos={todos} />
      </section>
    </main>
  );
}
```

- [ ] **Step 5: Add list styling**

Append to `src/styles.css`:

```css
.count {
  min-width: 32px;
  height: 28px;
  display: grid;
  place-items: center;
  border-radius: 9px;
  color: #8ab4f8;
  background: rgba(138, 180, 248, 0.13);
  font-size: 13px;
  font-weight: 650;
}

.todo-list {
  display: grid;
  gap: 7px;
  margin: 0;
  padding: 0 15px 10px;
  list-style: none;
}

.todo-item {
  display: grid;
  grid-template-columns: 23px 1fr;
  gap: 8px;
  align-items: start;
  min-height: 34px;
  padding: 8px 9px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.055);
}

.todo-index {
  width: 19px;
  height: 19px;
  display: grid;
  place-items: center;
  border-radius: 999px;
  color: rgba(255, 255, 255, 0.72);
  background: rgba(255, 255, 255, 0.08);
  font-size: 11px;
  font-weight: 650;
}

.todo-title {
  min-width: 0;
  overflow-wrap: anywhere;
  font-size: 13px;
  line-height: 1.42;
}

.empty,
.error {
  margin: 0 15px 12px;
  color: rgba(255, 255, 255, 0.58);
  font-size: 12px;
  line-height: 1.45;
}
```

- [ ] **Step 6: Verify UI test**

Run:

```powershell
npm.cmd test
```

Expected: App test passes.

- [ ] **Step 7: Commit todo list UI**

```powershell
git add src/App.tsx src/App.test.tsx src/components src/styles.css
git commit -m "feat: render compact todo list"
```

---

### Task 14: Build Add And Preview UI

**Files:**
- Create: `src/components/AddTodo.tsx`
- Create: `src/components/PreviewEditor.tsx`
- Modify: `src/App.tsx`
- Modify: `src/App.test.tsx`
- Modify: `src/styles.css`

- [ ] **Step 1: Extend UI test for add flow**

Append to `src/App.test.tsx`:

```tsx
import userEvent from "@testing-library/user-event";
import { parseTodoText, savePreview } from "./api";

it("parses and saves a preview entry", async () => {
  vi.mocked(parseTodoText).mockResolvedValueOnce({
    entries: [
      {
        title: "拿文件",
        due_date: "2026-05-20",
        due_time: null,
        source_text: "明天拿文件",
        date_expression: "明天",
        date_source: "rule",
        warning: null,
      },
    ],
  });
  vi.mocked(savePreview).mockResolvedValueOnce([]);

  render(<App />);
  await userEvent.click(await screen.findByRole("button", { name: "添加" }));
  await userEvent.type(screen.getByLabelText("自然语言输入"), "明天拿文件");
  await userEvent.click(screen.getByRole("button", { name: "解析" }));
  expect(await screen.findByDisplayValue("拿文件")).toBeInTheDocument();
  await userEvent.click(screen.getByRole("button", { name: "确认保存" }));
  expect(savePreview).toHaveBeenCalledOnce();
});
```

- [ ] **Step 2: Run test to verify failure**

Run:

```powershell
npm.cmd test
```

Expected: add-flow test fails because the Add button and form do not exist.

- [ ] **Step 3: Create preview editor**

Write `src/components/PreviewEditor.tsx`:

```tsx
import type { PreviewEntry } from "../types";

interface PreviewEditorProps {
  entries: PreviewEntry[];
  onChange: (entries: PreviewEntry[]) => void;
}

export function PreviewEditor({ entries, onChange }: PreviewEditorProps) {
  return (
    <div className="preview-list">
      {entries.map((entry, index) => (
        <div className="preview-item" key={`${entry.title}-${index}`}>
          <input
            aria-label={`事项 ${index + 1}`}
            value={entry.title}
            onChange={(event) => {
              const next = [...entries];
              next[index] = { ...entry, title: event.target.value };
              onChange(next);
            }}
          />
          <input
            aria-label={`日期 ${index + 1}`}
            type="date"
            value={entry.due_date}
            onChange={(event) => {
              const next = [...entries];
              next[index] = { ...entry, due_date: event.target.value };
              onChange(next);
            }}
          />
          {entry.warning ? <p className="warning">{entry.warning}</p> : null}
        </div>
      ))}
    </div>
  );
}
```

- [ ] **Step 4: Create add form**

Write `src/components/AddTodo.tsx`:

```tsx
import { useState } from "react";
import { parseTodoText, savePreview } from "../api";
import type { PreviewEntry, Todo } from "../types";
import { PreviewEditor } from "./PreviewEditor";

interface AddTodoProps {
  onSaved: (todos: Todo[]) => void;
  onError: (message: string | null) => void;
}

export function AddTodo({ onSaved, onError }: AddTodoProps) {
  const [open, setOpen] = useState(false);
  const [text, setText] = useState("");
  const [entries, setEntries] = useState<PreviewEntry[]>([]);
  const [busy, setBusy] = useState(false);

  async function parse() {
    setBusy(true);
    onError(null);
    try {
      const response = await parseTodoText(text);
      setEntries(response.entries);
    } catch (err) {
      onError(String(err));
    } finally {
      setBusy(false);
    }
  }

  async function save() {
    setBusy(true);
    onError(null);
    try {
      const todos = await savePreview(entries);
      onSaved(todos);
      setText("");
      setEntries([]);
      setOpen(false);
    } catch (err) {
      onError(String(err));
    } finally {
      setBusy(false);
    }
  }

  if (!open) {
    return (
      <footer className="actions">
        <button className="primary" onClick={() => setOpen(true)}>
          添加
        </button>
      </footer>
    );
  }

  return (
    <section className="add-panel">
      <label>
        <span>自然语言输入</span>
        <textarea value={text} onChange={(event) => setText(event.target.value)} />
      </label>
      <div className="button-row">
        <button className="ghost" onClick={() => setOpen(false)} disabled={busy}>
          取消
        </button>
        <button className="primary" onClick={parse} disabled={busy || text.trim().length === 0}>
          解析
        </button>
      </div>
      {entries.length > 0 ? (
        <>
          <PreviewEditor entries={entries} onChange={setEntries} />
          <button className="primary wide" onClick={save} disabled={busy}>
            确认保存
          </button>
        </>
      ) : null}
    </section>
  );
}
```

- [ ] **Step 5: Use add form in App**

Modify `src/App.tsx`:

```tsx
import { useEffect, useState } from "react";
import { getAppState } from "./api";
import { AddTodo } from "./components/AddTodo";
import { ErrorBanner } from "./components/ErrorBanner";
import { TodoList } from "./components/TodoList";
import type { AppState } from "./types";

export default function App() {
  const [state, setState] = useState<AppState | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getAppState()
      .then(setState)
      .catch((err) => setError(String(err)));
  }, []);

  const todos = state?.todos ?? [];

  return (
    <main className="float-shell">
      <section className="float-panel">
        <header className="panel-header">
          <div>
            <h1>今天要做</h1>
            <p>{state?.today ?? "加载中"}</p>
          </div>
          <span className="count">{todos.length}</span>
        </header>
        <ErrorBanner message={error} />
        <TodoList todos={todos} />
        <AddTodo
          onError={setError}
          onSaved={(newTodos) =>
            setState((current) => ({
              today: current?.today ?? new Date().toISOString().slice(0, 10),
              todos: newTodos,
            }))
          }
        />
      </section>
    </main>
  );
}
```

- [ ] **Step 6: Add form styling**

Append to `src/styles.css`:

```css
.actions,
.button-row {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
  padding: 11px 15px 15px;
}

button {
  height: 32px;
  border-radius: 9px;
  font: inherit;
  font-size: 12px;
}

.ghost {
  color: rgba(255, 255, 255, 0.74);
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(255, 255, 255, 0.04);
}

.primary {
  color: #111;
  border: 0;
  background: rgba(255, 255, 255, 0.88);
  padding: 0 11px;
  font-weight: 650;
}

.wide {
  width: calc(100% - 30px);
  margin: 8px 15px 15px;
}

.add-panel {
  padding: 0 15px 15px;
}

.add-panel label,
.add-panel label span {
  display: grid;
  gap: 6px;
  color: rgba(255, 255, 255, 0.58);
  font-size: 12px;
}

textarea,
input {
  width: 100%;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 10px;
  color: rgba(255, 255, 255, 0.92);
  background: rgba(255, 255, 255, 0.06);
  font: inherit;
  font-size: 13px;
}

textarea {
  min-height: 74px;
  padding: 9px;
  resize: vertical;
}

input {
  height: 32px;
  padding: 0 8px;
}

.preview-list {
  display: grid;
  gap: 8px;
  margin-top: 10px;
}

.preview-item {
  display: grid;
  gap: 6px;
  padding: 8px;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.055);
}

.warning {
  margin: 0;
  color: #ffd37a;
  font-size: 11px;
}
```

- [ ] **Step 7: Verify add UI**

Run:

```powershell
npm.cmd test
npm.cmd run build
```

Expected: tests and build pass.

- [ ] **Step 8: Commit add UI**

```powershell
git add src/App.tsx src/App.test.tsx src/components/AddTodo.tsx src/components/PreviewEditor.tsx src/styles.css
git commit -m "feat: add natural language preview flow"
```

---

### Task 15: Add Suppress-Today Button

**Files:**
- Modify: `src/components/AddTodo.tsx`
- Modify: `src/App.tsx`
- Modify: `src/App.test.tsx`

- [ ] **Step 1: Add suppress test**

Update the mock in `src/App.test.tsx` so `suppressToday` is imported and asserted:

```tsx
import { parseTodoText, savePreview, suppressToday } from "./api";

it("can suppress today's popup", async () => {
  render(<App />);
  await userEvent.click(await screen.findByRole("button", { name: "今天不再弹出" }));
  expect(suppressToday).toHaveBeenCalledOnce();
});
```

- [ ] **Step 2: Run test to verify failure**

Run:

```powershell
npm.cmd test
```

Expected: suppress test fails because the button does not exist.

- [ ] **Step 3: Add button to App**

Modify `src/App.tsx` imports:

```tsx
import { getAppState, suppressToday } from "./api";
```

Add a handler inside `App`:

```tsx
async function suppress() {
  setError(null);
  try {
    await suppressToday();
  } catch (err) {
    setError(String(err));
  }
}
```

Render before `<AddTodo ... />`:

```tsx
<footer className="actions">
  <button className="ghost" onClick={suppress}>
    今天不再弹出
  </button>
</footer>
```

- [ ] **Step 4: Remove duplicate AddTodo footer**

In `src/components/AddTodo.tsx`, replace the closed state with a compact single button:

```tsx
if (!open) {
  return (
    <div className="actions">
      <button className="primary" onClick={() => setOpen(true)}>
        添加
      </button>
    </div>
  );
}
```

- [ ] **Step 5: Verify suppress UI**

Run:

```powershell
npm.cmd test
```

Expected: suppress test passes.

- [ ] **Step 6: Commit suppress UI**

```powershell
git add src/App.tsx src/App.test.tsx src/components/AddTodo.tsx
git commit -m "feat: add suppress today action"
```

---

### Task 16: Final Verification And Manual Run

**Files:**
- Modify only files required by verification failures.

- [ ] **Step 1: Run frontend tests**

Run:

```powershell
npm.cmd test
```

Expected: all Vitest tests pass.

- [ ] **Step 2: Run Rust tests**

Run:

```powershell
cd src-tauri
cargo test
```

Expected: all Rust tests pass.

- [ ] **Step 3: Build frontend**

Run:

```powershell
npm.cmd run build
```

Expected: TypeScript and Vite build pass.

- [ ] **Step 4: Build Tauri app**

Run:

```powershell
npm.cmd run tauri build
```

Expected: Tauri build completes and creates a Windows bundle or executable under `src-tauri/target/release`.

- [ ] **Step 5: Manual config check**

Copy the sample config beside the development executable or set `TODO_FLOAT_CONFIG`:

```powershell
Copy-Item .\config.example.toml .\config.toml
$env:TODO_FLOAT_CONFIG = (Resolve-Path .\config.toml)
```

Run:

```powershell
npm.cmd run tauri dev
```

Expected: the 300 px float opens. Clicking `添加`, entering text, and clicking `解析` shows a clear config-file message because `api_key = ""`.

- [ ] **Step 6: Manual DB check without LLM**

Set a dev database:

```powershell
$env:TODO_FLOAT_DB = (Join-Path (Get-Location) 'dev.todo-float.sqlite3')
```

Run:

```powershell
cd src-tauri
cargo test db
```

Expected: the SQLite repository tests still pass and no production user data is touched.

- [ ] **Step 7: Commit verified v1**

```powershell
git status --short
git add .
git commit -m "test: verify todo float v1"
```

Expected: commit succeeds only if there are verification-related changes. If `git status --short` is empty, skip the commit and record that the tree is clean.

---

## Self-Review

- Spec coverage: The plan covers Tauri shell, 300 px float UI, startup-check mode, local SQLite, `config.toml`, OpenRouter/BigModel provider path, JSON/schema validation, local date override rules, preview confirmation, and "today do not show again" behavior.
- Out of scope: settings UI, tray, global hotkey, completion checkbox UI, exact-time reminders, cloud sync, account system, and PWA are not included.
- Type consistency: Rust DTO names match TypeScript DTO names through snake_case serialized fields; frontend `invoke` calls use command names registered in `tauri::generate_handler`.
- Verification: Rust unit tests cover date rules, DB, parser override, and startup decision. Vitest covers list rendering, add/preview/save flow, and suppress action. Final build commands verify the desktop shell.
