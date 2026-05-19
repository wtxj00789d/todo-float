import { useState } from "react";

import { parseTodoText, savePreview } from "../api";
import type { PreviewEntry, Todo } from "../types";
import { PreviewEditor } from "./PreviewEditor";

interface AddTodoProps {
  onError: (message: string | null) => void;
  onSaved: (todos: Todo[]) => void;
}

export function AddTodo({ onError, onSaved }: AddTodoProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [text, setText] = useState("");
  const [entries, setEntries] = useState<PreviewEntry[]>([]);
  const [isParsing, setIsParsing] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const trimmedText = text.trim();
  const isBusy = isParsing || isSaving;

  const close = () => {
    setIsOpen(false);
    setText("");
    setEntries([]);
  };

  const parse = async () => {
    if (!trimmedText || isBusy) {
      return;
    }

    setIsParsing(true);
    onError(null);

    try {
      const response = await parseTodoText(trimmedText);
      setEntries(response.entries);
    } catch (reason: unknown) {
      onError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setIsParsing(false);
    }
  };

  const save = async () => {
    if (entries.length === 0 || isBusy) {
      return;
    }

    setIsSaving(true);
    onError(null);

    try {
      const todos = await savePreview(entries);
      onSaved(todos);
      close();
    } catch (reason: unknown) {
      onError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setIsSaving(false);
    }
  };

  if (!isOpen) {
    return (
      <footer className="actions">
        <button className="primary-button" type="button" onClick={() => setIsOpen(true)}>
          添加
        </button>
      </footer>
    );
  }

  return (
    <section className="add-panel">
      <label className="field-label" htmlFor="todo-natural-input">
        自然语言输入
      </label>
      <textarea
        className="text-area"
        disabled={isBusy}
        id="todo-natural-input"
        onChange={(event) => {
          setText(event.target.value);
          setEntries([]);
        }}
        rows={3}
        value={text}
      />
      <div className="actions">
        <button className="ghost-button" disabled={isBusy} type="button" onClick={close}>
          取消
        </button>
        <button className="primary-button" disabled={!trimmedText || isBusy} type="button" onClick={parse}>
          解析
        </button>
      </div>
      {entries.length > 0 ? (
        <>
          <PreviewEditor entries={entries} onChange={setEntries} />
          <div className="actions">
            <button className="primary-button" disabled={isBusy || entries.length === 0} type="button" onClick={save}>
              确认保存
            </button>
          </div>
        </>
      ) : null}
    </section>
  );
}
