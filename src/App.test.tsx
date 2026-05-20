import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import App from "./App";
import { getAppState, openVoiceInput, parseTodoText, savePreview, suppressToday } from "./api";
import type { AppState, PreviewEntry, Todo } from "./types";

vi.mock("./api", () => ({
  getAppState: vi.fn(),
  parseTodoText: vi.fn(),
  savePreview: vi.fn(),
  suppressToday: vi.fn(),
  openVoiceInput: vi.fn(),
}));

const getAppStateMock = vi.mocked(getAppState);
const parseTodoTextMock = vi.mocked(parseTodoText);
const savePreviewMock = vi.mocked(savePreview);
const suppressTodayMock = vi.mocked(suppressToday);
const openVoiceInputMock = vi.mocked(openVoiceInput);

const todo: Todo = {
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
};

const previewEntry: PreviewEntry = {
  title: "拿文件",
  due_date: "2026-05-19",
  due_time: null,
  source_text: "明天拿文件",
  date_expression: "明天",
  date_source: "rule",
  warning: null,
};

describe("App", () => {
  beforeEach(() => {
    vi.resetAllMocks();
  });

  afterEach(() => {
    cleanup();
  });

  it("renders today's todo title and count", async () => {
    const state: AppState = {
      today: "2026-05-19",
      todos: [todo],
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

    expect(document.querySelector(".empty")).not.toBeInTheDocument();
    expect(document.querySelector(".loading")).toHaveTextContent("加载中...");
    expect(screen.queryByRole("button", { name: "添加" })).not.toBeInTheDocument();

    resolveState({ today: "2026-05-19", todos: [] });

    expect(await screen.findByText("今天没有事项")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "添加" })).toBeInTheDocument();
  });

  it("suppresses today's popup from the loaded state", async () => {
    const user = userEvent.setup();
    getAppStateMock.mockResolvedValueOnce({ today: "2026-05-19", todos: [] });
    suppressTodayMock.mockResolvedValueOnce();

    render(<App />);

    const suppressButton = await screen.findByRole("button", { name: "今天不再弹出" });
    await user.click(suppressButton);

    expect(suppressTodayMock).toHaveBeenCalledOnce();
  });

  it("parses and saves a natural language todo preview", async () => {
    const user = userEvent.setup();
    getAppStateMock.mockResolvedValueOnce({ today: "2026-05-19", todos: [] });
    parseTodoTextMock.mockResolvedValueOnce({ entries: [previewEntry] });
    savePreviewMock.mockResolvedValueOnce([todo]);

    render(<App />);

    await screen.findByText("今天没有事项");
    await user.click(screen.getByRole("button", { name: "添加" }));
    await user.type(screen.getByLabelText("自然语言输入"), "明天拿文件");
    await user.click(screen.getByRole("button", { name: "解析" }));

    const titleInput = await screen.findByLabelText("事项 1");
    const dateInput = screen.getByLabelText("日期 1");

    expect(titleInput).toHaveValue("拿文件");

    await user.clear(titleInput);
    await user.type(titleInput, "送文件");
    await user.clear(dateInput);
    await user.type(dateInput, "2026-05-20");

    await user.click(screen.getByRole("button", { name: "确认保存" }));

    expect(savePreviewMock).toHaveBeenCalledWith([{ ...previewEntry, title: "送文件", due_date: "2026-05-20" }]);
  });

  it("opens Windows voice input from the natural language field", async () => {
    const user = userEvent.setup();
    getAppStateMock.mockResolvedValueOnce({ today: "2026-05-19", todos: [] });
    openVoiceInputMock.mockResolvedValueOnce();

    render(<App />);

    await screen.findByText("今天没有事项");
    await user.click(screen.getByRole("button", { name: "添加" }));
    await user.click(screen.getByRole("button", { name: "语音" }));

    expect(screen.getByLabelText("自然语言输入")).toHaveFocus();
    expect(openVoiceInputMock).toHaveBeenCalledOnce();
  });
});
