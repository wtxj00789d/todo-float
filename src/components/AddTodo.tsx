import { useEffect, useRef, useState } from "react";

import { openVoiceInput, parseTodoText, savePreview } from "../api";
import type { PreviewEntry, Todo } from "../types";
import { PreviewEditor } from "./PreviewEditor";

interface AddTodoProps {
  initialOpen?: boolean;
  onError: (message: string | null) => void;
  onSaved: (todos: Todo[]) => void;
}

export function AddTodo({ initialOpen = false, onError, onSaved }: AddTodoProps) {
  const [isOpen, setIsOpen] = useState(initialOpen);
  const [shouldOpenVoice, setShouldOpenVoice] = useState(false);
  const [text, setText] = useState("");
  const [entries, setEntries] = useState<PreviewEntry[]>([]);
  const [isParsing, setIsParsing] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const textAreaRef = useRef<HTMLTextAreaElement | null>(null);
  const trimmedText = text.trim();
  const isBusy = isParsing || isSaving;

  useEffect(() => {
    if (!isOpen) {
      return;
    }

    textAreaRef.current?.focus();

    if (!shouldOpenVoice) {
      return;
    }

    setShouldOpenVoice(false);
    onError(null);
    Promise.resolve(openVoiceInput()).catch((reason: unknown) => {
      onError(reason instanceof Error ? reason.message : String(reason));
    });
  }, [isOpen, onError, shouldOpenVoice]);

  const close = () => {
    setIsOpen(false);
    setShouldOpenVoice(false);
    setText("");
    setEntries([]);
    onError(null);
  };

  const openWithVoice = () => {
    setIsOpen(true);
    setShouldOpenVoice(true);
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

  const startVoiceInput = async () => {
    textAreaRef.current?.focus();
    onError(null);
    try {
      await openVoiceInput();
    } catch (reason: unknown) {
      onError(reason instanceof Error ? reason.message : String(reason));
    }
  };

  if (!isOpen) {
    return (
      <footer className="actions">
        <button className="primary-button" type="button" onClick={openWithVoice}>
          添加
        </button>
      </footer>
    );
  }

  return (
    <section className="add-panel">
      <div className="field-head">
        <label className="field-label" htmlFor="todo-natural-input">
          自然语言输入
        </label>
        <button
          aria-label="语音输入"
          className="ghost-button icon-button voice-button"
          disabled={isBusy}
          title="语音输入"
          type="button"
          onClick={startVoiceInput}
        >
          <svg aria-hidden="true" viewBox="0 0 24 24">
            <path d="M12 3.75a3.25 3.25 0 0 0-3.25 3.25v5a3.25 3.25 0 0 0 6.5 0V7A3.25 3.25 0 0 0 12 3.75Z" />
            <path d="M6.75 11.25a.75.75 0 0 1 .75.75 4.5 4.5 0 0 0 9 0 .75.75 0 0 1 1.5 0 6 6 0 0 1-5.25 5.95v1.3h2a.75.75 0 0 1 0 1.5h-5.5a.75.75 0 0 1 0-1.5h2v-1.3A6 6 0 0 1 6 12a.75.75 0 0 1 .75-.75Z" />
          </svg>
        </button>
      </div>
      <textarea
        className="text-area"
        disabled={isBusy}
        id="todo-natural-input"
        onChange={(event) => {
          setText(event.target.value);
          setEntries([]);
        }}
        ref={textAreaRef}
        rows={3}
        value={text}
      />
      <div className="actions">
        <button className="ghost-button" disabled={isBusy} type="button" onClick={close}>
          取消
        </button>
        <button className="primary-button" disabled={!trimmedText || isBusy} type="button" onClick={parse}>
          预览
        </button>
      </div>
      {entries.length > 0 ? (
        <div className="preview-area">
          <PreviewEditor entries={entries} onChange={setEntries} />
          <div className="actions preview-actions">
            <button className="primary-button" disabled={isBusy || entries.length === 0} type="button" onClick={save}>
              确认保存
            </button>
          </div>
        </div>
      ) : null}
    </section>
  );
}
