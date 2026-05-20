import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";

import {
  getAppState,
  openVoiceInput,
  parseTodoText,
  savePreview,
  suppressToday,
} from "./api";
import type { PreviewEntry } from "./types";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const invokeMock = vi.mocked(invoke);

describe("api", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("parses todo text through the Tauri command", async () => {
    invokeMock.mockResolvedValueOnce({ entries: [] });

    await parseTodoText("x");

    expect(invokeMock).toHaveBeenCalledWith("parse_todo_text", {
      request: { text: "x" },
    });
  });

  it("saves preview entries through the Tauri command", async () => {
    const entries: PreviewEntry[] = [
      {
        title: "Review notes",
        due_date: "2026-05-19",
        due_time: null,
        source_text: "Review notes today",
        date_expression: "today",
        date_source: "rule",
        warning: null,
      },
    ];
    invokeMock.mockResolvedValueOnce([]);

    await savePreview(entries);

    expect(invokeMock).toHaveBeenCalledWith("save_preview", {
      request: { entries },
    });
  });

  it("loads app state through the Tauri command", async () => {
    invokeMock.mockResolvedValueOnce({ today: "2026-05-19", todos: [] });

    await getAppState();

    expect(invokeMock).toHaveBeenCalledWith("get_app_state");
  });

  it("suppresses today through the Tauri command", async () => {
    invokeMock.mockResolvedValueOnce(undefined);

    await suppressToday();

    expect(invokeMock).toHaveBeenCalledWith("suppress_today");
  });

  it("opens Windows voice input through the Tauri command", async () => {
    invokeMock.mockResolvedValueOnce(undefined);

    await openVoiceInput();

    expect(invokeMock).toHaveBeenCalledWith("open_voice_input");
  });
});
