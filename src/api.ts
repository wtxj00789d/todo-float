import { invoke } from "@tauri-apps/api/core";

import type { AppState, ParseResponse, PreviewEntry, Todo } from "./types";

export function getAppState(): Promise<AppState> {
  return invoke<AppState>("get_app_state");
}

export function parseTodoText(text: string): Promise<ParseResponse> {
  return invoke<ParseResponse>("parse_todo_text", { request: { text } });
}

export function savePreview(entries: PreviewEntry[]): Promise<Todo[]> {
  return invoke<Todo[]>("save_preview", { request: { entries } });
}

export function suppressToday(): Promise<void> {
  return invoke<void>("suppress_today");
}
