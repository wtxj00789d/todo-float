import { type MouseEvent, useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

import { getAppState, suppressToday } from "./api";
import { AddTodo } from "./components/AddTodo";
import { ErrorBanner } from "./components/ErrorBanner";
import { TodoList } from "./components/TodoList";
import type { AppState, Todo } from "./types";

export default function App() {
  const [state, setState] = useState<AppState | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isSuppressingToday, setIsSuppressingToday] = useState(false);
  const todos = state?.todos ?? [];

  const handleSaved = (savedTodos: Todo[]) => {
    setState((currentState) => (currentState === null ? currentState : { ...currentState, todos: savedTodos }));
    setError(null);
  };

  const handleSuppressToday = async () => {
    if (isSuppressingToday) {
      return;
    }

    setIsSuppressingToday(true);
    setError(null);

    try {
      await suppressToday();
      setError(null);
    } catch (reason: unknown) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setIsSuppressingToday(false);
    }
  };

  const handleHeaderMouseDown = async (event: MouseEvent<HTMLElement>) => {
    if (event.button !== 0) {
      return;
    }

    try {
      await getCurrentWindow().startDragging();
    } catch {
      // Dragging is only available in the Tauri runtime.
    }
  };

  useEffect(() => {
    let isMounted = true;

    getAppState()
      .then((appState) => {
        if (!isMounted) {
          return;
        }

        setState(appState);
        setError(null);
      })
      .catch((reason: unknown) => {
        if (!isMounted) {
          return;
        }

        setError(reason instanceof Error ? reason.message : String(reason));
      });

    return () => {
      isMounted = false;
    };
  }, []);

  return (
    <main className="float-shell">
      <section className="float-panel">
        <header className="panel-header" onMouseDown={handleHeaderMouseDown}>
          <div>
            <h1>今天要做</h1>
            <p>{state?.today ?? "加载中..."}</p>
          </div>
          <span className="count">{todos.length}</span>
        </header>
        <ErrorBanner message={error} />
        {state === null ? null : (
          <div className="panel-actions">
            <button className="ghost-button compact-button" disabled={isSuppressingToday} type="button" onClick={handleSuppressToday}>
              今天不再弹出
            </button>
          </div>
        )}
        {state === null ? (
          error ? null : <p className="loading">加载中...</p>
        ) : (
          <TodoList todos={todos} />
        )}
        {state === null ? null : <AddTodo onError={setError} onSaved={handleSaved} />}
      </section>
    </main>
  );
}
