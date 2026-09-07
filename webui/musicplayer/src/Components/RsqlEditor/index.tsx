import { FC, useCallback, useMemo, useRef, useState } from "react";
import cn from "../UI/cn";
import { TRACK_FIELDS } from "./fields";
import { completionAt, tokenize, type TokenKind } from "./rsql";

/**
 * A filter input with syntax highlighting and completion.
 *
 * Highlighting is a coloured `<pre>` sitting exactly under a transparent
 * `<textarea>` — the browser draws the caret and handles selection, while the
 * layer beneath supplies the colour. Both must lay out identically down to the
 * letter spacing or the colours drift away from the caret, which is what
 * `LAYOUT` below is for. This is the same construction as the desktop's
 * `FilterBox`, which stacks a coloured run layer under an invisible
 * `TextInput`.
 */

/** The metrics the two layers must agree on, to the pixel. */
const LAYOUT =
  "m-0 px-3 py-[10px] font-mono text-[13px] leading-normal whitespace-pre-wrap break-words tracking-normal";

/**
 * Colours by role, from the skin's syntax tokens — the same five the desktop
 * reads out of its skin .toml. `unknown-field` is the loud one on purpose: a
 * mistyped field is the mistake people actually make, and it is otherwise
 * invisible.
 */
const COLORS: Record<TokenKind, string> = {
  field: "text-syntax-field",
  "unknown-field": "text-syntax-error",
  op: "text-syntax-operator",
  value: "text-syntax-value",
  string: "text-syntax-value",
  logic: "text-syntax-logic",
  paren: "text-syntax-operator",
  space: "",
  error: "text-syntax-error",
};

export type RsqlEditorProps = {
  value: string;
  onChange: (value: string) => void;
  /** Called when the value settles — blur, or Enter with no suggestion open. */
  onCommit?: (value: string) => void;
  placeholder?: string;
  /** Validation message from the form; borders the editor in the error hue. */
  error?: string;
};

const RsqlEditor: FC<RsqlEditorProps> = ({
  value,
  onChange,
  onCommit,
  placeholder,
  error,
}) => {
  const ref = useRef<HTMLTextAreaElement>(null);
  const [caret, setCaret] = useState(0);
  const [selected, setSelected] = useState(0);
  const [open, setOpen] = useState(false);

  const tokens = useMemo(() => tokenize(value, TRACK_FIELDS), [value]);
  const completion = useMemo(() => completionAt(value, caret), [value, caret]);
  const suggestions = useMemo(
    () => (open ? completion.suggestions.slice(0, 8) : []),
    [open, completion]
  );

  const accept = useCallback(
    (text: string) => {
      const next =
        value.slice(0, completion.from) + text + value.slice(completion.to);
      const at = completion.from + text.length;
      onChange(next);
      setOpen(false);
      // Restoring the caret has to wait for React to write the new value, or
      // the browser puts it back at the end.
      requestAnimationFrame(() => {
        const node = ref.current;
        if (!node) return;
        node.focus();
        node.setSelectionRange(at, at);
        setCaret(at);
      });
    },
    [completion, onChange, value]
  );

  const sync = (node: HTMLTextAreaElement) => setCaret(node.selectionStart ?? 0);

  return (
    <div className="relative">
      <div
        className={cn(
          "relative overflow-hidden rounded-control border bg-raised",
          error ? "border-syntax-error" : "border-line focus-within:border-accent"
        )}
      >
        <pre aria-hidden className={cn(LAYOUT, "min-h-10 text-fg")}>
          {tokens.map((token, i) => (
            <span
              key={i}
              className={cn(
                COLORS[token.kind],
                (token.kind === "unknown-field" || token.kind === "error") &&
                  "underline decoration-wavy underline-offset-[3px]",
                token.kind === "unknown-field" && "font-bold"
              )}
            >
              {token.text}
            </span>
          ))}
          {/* A trailing newline keeps the last line's height when the value
              ends in one, so the overlay does not shrink under the caret. */}
          {value.endsWith("\n") ? "\n" : ""}
        </pre>
        <textarea
          ref={ref}
          rows={1}
          spellCheck={false}
          autoCapitalize="off"
          autoCorrect="off"
          value={value}
          placeholder={placeholder}
          aria-invalid={!!error}
          style={{ caretColor: "var(--text)" }}
          className={cn(
            LAYOUT,
            "absolute inset-0 size-full resize-none border-none bg-transparent text-transparent outline-none",
            "placeholder:text-muted"
          )}
          onChange={(e) => {
            onChange(e.target.value);
            sync(e.target);
            setOpen(true);
            setSelected(0);
          }}
          onClick={(e) => sync(e.currentTarget)}
          onKeyUp={(e) => sync(e.currentTarget)}
          onBlur={() => {
            // A click on a suggestion blurs the textarea first, so closing has
            // to wait or the click never lands.
            window.setTimeout(() => setOpen(false), 120);
            onCommit?.(value);
          }}
          onKeyDown={(e) => {
            if (!suggestions.length) {
              if (e.key === "Enter") {
                e.preventDefault();
                onCommit?.(value);
              }
              return;
            }
            if (e.key === "ArrowDown") {
              e.preventDefault();
              setSelected((s) => (s + 1) % suggestions.length);
            } else if (e.key === "ArrowUp") {
              e.preventDefault();
              setSelected(
                (s) => (s - 1 + suggestions.length) % suggestions.length
              );
            } else if (e.key === "Enter" || e.key === "Tab") {
              e.preventDefault();
              accept(suggestions[selected].text);
            } else if (e.key === "Escape") {
              setOpen(false);
            }
          }}
        />
      </div>

      {error && <p className="mt-1 text-[11px] text-syntax-error">{error}</p>}

      {suggestions.length > 0 && (
        <ul className="absolute inset-x-0 z-30 mt-1 max-h-[180px] overflow-y-auto rounded-control border border-line bg-panel p-1 shadow-[0_8px_24px_rgba(0,0,0,0.4)]">
          {suggestions.map((suggestion, i) => (
            <li key={suggestion.text}>
              <button
                type="button"
                // `mouseDown` rather than `click`: the textarea's blur would
                // otherwise close the list before the click resolved.
                onMouseDown={(e) => {
                  e.preventDefault();
                  accept(suggestion.text);
                }}
                className={cn(
                  "flex w-full items-baseline gap-[10px] rounded px-2 py-[5px] text-left",
                  i === selected ? "bg-selected" : "hover:bg-hover"
                )}
              >
                <span
                  className={cn(
                    "font-mono text-[13px]",
                    i === selected ? "text-accent" : "text-fg"
                  )}
                >
                  {suggestion.text}
                </span>
                {suggestion.hint && (
                  <span className="ml-auto text-[11px] text-muted">
                    {suggestion.hint}
                  </span>
                )}
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
};

export default RsqlEditor;
