import { useEffect, useState } from "react";

import { getAppState } from "./api";
import { ErrorBanner } from "./components/ErrorBanner";
import { TodoList } from "./components/TodoList";
import type { AppState } from "./types";

export default function App() {
  const [state, setState] = useState<AppState | null>(null);
  const [error, setError] = useState<string | null>(null);
  const todos = state?.todos ?? [];

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
        <header className="panel-header">
          <div>
            <h1>今天要做</h1>
            <p>{state?.today ?? "加载中..."}</p>
          </div>
          <span className="count">{todos.length}</span>
        </header>
        <ErrorBanner message={error} />
        {state === null ? (
          error ? null : <p className="loading">加载中...</p>
        ) : (
          <TodoList todos={todos} />
        )}
      </section>
    </main>
  );
}
