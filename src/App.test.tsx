import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import App from "./App";
import { exitApp, getAppState, openVoiceInput, parseTodoText, savePreview, suppressToday } from "./api";
import type { AppState, LaunchMode, PreviewEntry, Todo } from "./types";

const { minimizeMock, setAlwaysOnTopMock, startDraggingMock } = vi.hoisted(() => ({
  minimizeMock: vi.fn(),
  setAlwaysOnTopMock: vi.fn(),
  startDraggingMock: vi.fn(),
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    minimize: minimizeMock,
    setAlwaysOnTop: setAlwaysOnTopMock,
    startDragging: startDraggingMock,
  }),
}));

vi.mock("./api", () => ({
  exitApp: vi.fn(),
  getAppState: vi.fn(),
  parseTodoText: vi.fn(),
  savePreview: vi.fn(),
  suppressToday: vi.fn(),
  openVoiceInput: vi.fn(),
}));

const exitAppMock = vi.mocked(exitApp);
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

const secondPreviewEntry: PreviewEntry = {
  ...previewEntry,
  title: "找凯莉",
  source_text: "明天找凯莉",
};

function appState(launchMode: LaunchMode = "startup_check", todos: Todo[] = []): AppState {
  return {
    today: "2026-05-19",
    todos,
    launch_mode: launchMode,
  };
}

describe("App", () => {
  beforeEach(() => {
    vi.resetAllMocks();
    exitAppMock.mockResolvedValue(undefined);
    minimizeMock.mockResolvedValue(undefined);
    openVoiceInputMock.mockResolvedValue(undefined);
    setAlwaysOnTopMock.mockResolvedValue(undefined);
    startDraggingMock.mockResolvedValue(undefined);
  });

  afterEach(() => {
    cleanup();
  });

  it("renders today's todo title and count", async () => {
    getAppStateMock.mockResolvedValueOnce(appState("startup_check", [todo]));

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

    resolveState(appState());

    expect(await screen.findByText("今天没有事项")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "添加" })).toBeInTheDocument();
  });

  it("suppresses today's popup and exits the app from the loaded state", async () => {
    const user = userEvent.setup();
    getAppStateMock.mockResolvedValueOnce(appState());
    suppressTodayMock.mockResolvedValueOnce();

    render(<App />);

    const suppressButton = await screen.findByRole("button", { name: "今日不再弹出" });
    await user.click(suppressButton);

    expect(suppressTodayMock).toHaveBeenCalledOnce();
    expect(exitAppMock).toHaveBeenCalledOnce();
  });

  it("starts native window dragging from the header", async () => {
    getAppStateMock.mockResolvedValueOnce(appState());

    render(<App />);

    await screen.findByText("2026-05-19");
    fireEvent.mouseDown(document.querySelector(".panel-header") as Element, { button: 0 });

    expect(startDraggingMock).toHaveBeenCalledOnce();
  });

  it("toggles always-on-top from the pin button", async () => {
    const user = userEvent.setup();
    getAppStateMock.mockResolvedValueOnce(appState());

    render(<App />);

    await user.click(await screen.findByRole("button", { name: "置顶" }));
    expect(setAlwaysOnTopMock).toHaveBeenCalledWith(true);

    await user.click(screen.getByRole("button", { name: "取消置顶" }));
    expect(setAlwaysOnTopMock).toHaveBeenCalledWith(false);
  });

  it("minimizes and closes from title controls", async () => {
    const user = userEvent.setup();
    getAppStateMock.mockResolvedValueOnce(appState());

    render(<App />);

    await screen.findByText("2026-05-19");
    await user.click(screen.getByRole("button", { name: "最小化" }));
    await user.click(screen.getByRole("button", { name: "关闭" }));

    expect(minimizeMock).toHaveBeenCalledOnce();
    expect(exitAppMock).toHaveBeenCalledOnce();
  });

  it("opens manual launches directly in the natural language field without voice input", async () => {
    getAppStateMock.mockResolvedValueOnce(appState("manual"));

    render(<App />);

    const textArea = await screen.findByLabelText("自然语言输入");

    await waitFor(() => expect(textArea).toHaveFocus());
    expect(screen.queryByRole("button", { name: "添加" })).not.toBeInTheDocument();
    expect(openVoiceInputMock).not.toHaveBeenCalled();
  });

  it("parses and saves a natural language todo preview", async () => {
    const user = userEvent.setup();
    getAppStateMock.mockResolvedValueOnce(appState());
    parseTodoTextMock.mockResolvedValueOnce({ entries: [previewEntry] });
    savePreviewMock.mockResolvedValueOnce([todo]);

    render(<App />);

    await screen.findByText("今天没有事项");
    await user.click(screen.getByRole("button", { name: "添加" }));
    await user.type(screen.getByLabelText("自然语言输入"), "明天拿文件");
    await user.click(screen.getByRole("button", { name: "预览" }));

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

  it("opens Windows voice input when adding from the startup popup", async () => {
    const user = userEvent.setup();
    getAppStateMock.mockResolvedValueOnce(appState());

    render(<App />);

    await screen.findByText("今天没有事项");
    await user.click(screen.getByRole("button", { name: "添加" }));

    expect(screen.getByLabelText("自然语言输入")).toHaveFocus();
    expect(openVoiceInputMock).toHaveBeenCalledOnce();
  });

  it("opens Windows voice input from the mic button", async () => {
    const user = userEvent.setup();
    getAppStateMock.mockResolvedValueOnce(appState("manual"));

    render(<App />);

    await screen.findByLabelText("自然语言输入");
    await user.click(screen.getByRole("button", { name: "语音输入" }));

    expect(screen.getByLabelText("自然语言输入")).toHaveFocus();
    expect(openVoiceInputMock).toHaveBeenCalledOnce();
  });

  it("keeps the save action in a dedicated preview area for multiple entries", async () => {
    const user = userEvent.setup();
    getAppStateMock.mockResolvedValueOnce(appState());
    parseTodoTextMock.mockResolvedValueOnce({ entries: [previewEntry, secondPreviewEntry] });

    render(<App />);

    await screen.findByText("今天没有事项");
    await user.click(screen.getByRole("button", { name: "添加" }));
    await user.type(screen.getByLabelText("自然语言输入"), "明天拿文件和找凯莉");
    await user.click(screen.getByRole("button", { name: "预览" }));

    expect(await screen.findByLabelText("事项 2")).toHaveValue("找凯莉");
    expect(screen.getByRole("button", { name: "确认保存" }).parentElement).toHaveClass("preview-actions");
  });
});
