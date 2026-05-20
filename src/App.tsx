import { type MouseEvent, useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

import { exitApp, getAppState, suppressToday } from "./api";
import { AddTodo } from "./components/AddTodo";
import { ErrorBanner } from "./components/ErrorBanner";
import { TodoList } from "./components/TodoList";
import type { AppState, Todo } from "./types";

export default function App() {
  const [state, setState] = useState<AppState | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isSuppressingToday, setIsSuppressingToday] = useState(false);
  const [isPinned, setIsPinned] = useState(false);
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
      await exitApp();
      setError(null);
    } catch (reason: unknown) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setIsSuppressingToday(false);
    }
  };

  const handleTogglePin = async () => {
    const nextPinned = !isPinned;

    try {
      await getCurrentWindow().setAlwaysOnTop(nextPinned);
      setIsPinned(nextPinned);
      setError(null);
    } catch (reason: unknown) {
      setError(reason instanceof Error ? reason.message : String(reason));
    }
  };

  const handleMinimize = async () => {
    try {
      await getCurrentWindow().minimize();
      setError(null);
    } catch (reason: unknown) {
      setError(reason instanceof Error ? reason.message : String(reason));
    }
  };

  const handleClose = async () => {
    try {
      await exitApp();
    } catch (reason: unknown) {
      setError(reason instanceof Error ? reason.message : String(reason));
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

  const stopHeaderDrag = (event: MouseEvent<HTMLElement>) => {
    event.stopPropagation();
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
          <div className="header-right" onMouseDown={stopHeaderDrag}>
            <span className="count">{todos.length}</span>
            <div className="window-controls" aria-label="窗口控制">
              <button
                aria-pressed={isPinned}
                className={`window-button ${isPinned ? "active" : ""}`}
                title={isPinned ? "取消置顶" : "置顶"}
                type="button"
                onClick={handleTogglePin}
              >
                <span className="sr-only">{isPinned ? "取消置顶" : "置顶"}</span>
                <svg aria-hidden="true" viewBox="0 0 24 24">
                  <path d="M8.25 3.75h7.5v1.5h-1.2l.72 5.22 3.23 2.28v1.5H12.75v6h-1.5v-6H5.5v-1.5l3.23-2.28.72-5.22h-1.2v-1.5Zm2.72 1.5-.81 5.88-2.28 1.62h8.24l-2.28-1.62-.81-5.88h-2.06Z" />
                </svg>
              </button>
              <button className="window-button" title="最小化" type="button" onClick={handleMinimize}>
                <span className="sr-only">最小化</span>
                <svg aria-hidden="true" viewBox="0 0 24 24">
                  <path d="M5 11.25h14v1.5H5v-1.5Z" />
                </svg>
              </button>
              <button className="window-button close-button" title="关闭" type="button" onClick={handleClose}>
                <span className="sr-only">关闭</span>
                <svg aria-hidden="true" viewBox="0 0 24 24">
                  <path d="m6.53 5.47 5.47 5.47 5.47-5.47 1.06 1.06L13.06 12l5.47 5.47-1.06 1.06L12 13.06l-5.47 5.47-1.06-1.06L10.94 12 5.47 6.53l1.06-1.06Z" />
                </svg>
              </button>
            </div>
          </div>
        </header>
        <ErrorBanner message={error} />
        {state === null ? null : (
          <div className="panel-actions">
            <button className="ghost-button compact-button" disabled={isSuppressingToday} type="button" onClick={handleSuppressToday}>
              今日不再弹出
            </button>
          </div>
        )}
        {state === null ? (
          error ? null : <p className="loading">加载中...</p>
        ) : (
          <TodoList todos={todos} />
        )}
        {state === null ? null : <AddTodo initialOpen={state.launch_mode === "manual"} onError={setError} onSaved={handleSaved} />}
      </section>
    </main>
  );
}
