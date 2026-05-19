import { useEffect, useState } from "react";

import { getAppState } from "./api";
import type { AppState } from "./types";

export default function App() {
  const [state, setState] = useState<AppState | null>(null);
  const [error, setError] = useState<string | null>(null);

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
            <p>{state?.today ?? "加载中"}</p>
          </div>
          <span>{state?.todos.length ?? 0}</span>
        </header>
        {error ? <p className="error">{error}</p> : null}
      </section>
    </main>
  );
}
