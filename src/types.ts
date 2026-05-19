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
