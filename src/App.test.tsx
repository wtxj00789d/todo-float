import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import App from "./App";
import { getAppState } from "./api";
import type { AppState } from "./types";

vi.mock("./api", () => ({
  getAppState: vi.fn(),
}));

const getAppStateMock = vi.mocked(getAppState);

describe("App", () => {
  beforeEach(() => {
    getAppStateMock.mockReset();
  });

  afterEach(() => {
    cleanup();
  });

  it("renders today's todo title and count", async () => {
    const state: AppState = {
      today: "2026-05-19",
      todos: [
        {
          id: "todo-1",
          title: "拿文件",
          due_date: "2026-05-19",
          due_time: null,
          source_text: null,
          date_expression: null,
          date_source: "rule",
          warning: null,
          created_at: "2026-05-19T00:00:00Z",
          updated_at: "2026-05-19T00:00:00Z",
          deleted_at: null,
          completed_at: null,
        },
      ],
    };
    getAppStateMock.mockResolvedValueOnce(state);

    render(<App />);

    expect(await screen.findByText("拿文件")).toBeInTheDocument();
    expect(document.querySelector(".count")).toHaveTextContent("1");
  });

  it("does not show the empty state before loading resolves", async () => {
    let resolveState: (state: AppState) => void = () => {};
    const statePromise = new Promise<AppState>((resolve) => {
      resolveState = resolve;
    });
    getAppStateMock.mockReturnValueOnce(statePromise);

    render(<App />);

    expect(screen.queryByText("今天没有事项")).not.toBeInTheDocument();
    expect(document.querySelector(".loading")).toHaveTextContent("加载中...");

    resolveState({ today: "2026-05-19", todos: [] });

    expect(await screen.findByText("今天没有事项")).toBeInTheDocument();
  });
});
