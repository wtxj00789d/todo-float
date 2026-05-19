import type { PreviewEntry } from "../types";

interface PreviewEditorProps {
  entries: PreviewEntry[];
  onChange: (entries: PreviewEntry[]) => void;
}

export function PreviewEditor({ entries, onChange }: PreviewEditorProps) {
  const updateEntry = (index: number, patch: Partial<PreviewEntry>) => {
    onChange(entries.map((entry, entryIndex) => (entryIndex === index ? { ...entry, ...patch } : entry)));
  };

  return (
    <div className="preview-list">
      {entries.map((entry, index) => (
        <div className="preview-item" key={`${entry.source_text}-${index}`}>
          <input
            aria-label={`事项 ${index + 1}`}
            className="text-input"
            value={entry.title}
            onChange={(event) => updateEntry(index, { title: event.target.value })}
          />
          <input
            aria-label={`日期 ${index + 1}`}
            className="date-input"
            type="date"
            value={entry.due_date}
            onChange={(event) => updateEntry(index, { due_date: event.target.value })}
          />
          {entry.warning ? <p className="warning">{entry.warning}</p> : null}
        </div>
      ))}
    </div>
  );
}
