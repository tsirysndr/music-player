import styled from "@emotion/styled";
import { FC, useCallback, useMemo, useRef, useState } from "react";
import { completionAt, tokenize, type TokenKind } from "./rsql";
import { TRACK_FIELDS } from "./fields";

/**
 * A filter input with syntax highlighting and completion.
 *
 * Highlighting is a coloured `<pre>` sitting exactly under a transparent
 * `<textarea>` — the browser draws the caret and handles selection, while the
 * layer beneath supplies the colour. Both must lay out identically down to the
 * letter spacing or the colours drift away from the caret, which is what the
 * shared block below is for.
 */

const Frame = styled.div`
  position: relative;
`;

const Editor = styled.div`
  position: relative;
  border-radius: 8px;
  border: 1px solid ${(props) => props.theme.colors.separator};
  background: ${(props) => props.theme.colors.secondaryBackground};
  overflow: hidden;

  &:focus-within {
    border-color: #ab28fc;
  }
`;

/* The overlay and the textarea must lay out identically or the coloured text
   drifts away from the caret. */
const shared = `
  margin: 0;
  padding: 10px 12px;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 13px;
  line-height: 1.5;
  white-space: pre-wrap;
  overflow-wrap: break-word;
  letter-spacing: normal;
  tab-size: 2;
`;

const Highlight = styled.pre`
  ${shared}
  min-height: 40px;
  pointer-events: none;
  color: ${(props) => props.theme.colors.text};
`;

const Input = styled.textarea`
  ${shared}
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  border: none;
  outline: none;
  resize: none;
  background: transparent;
  color: transparent;
  caret-color: ${(props) => props.theme.colors.text};
`;

const Suggestions = styled.ul`
  position: absolute;
  z-index: 30;
  left: 0;
  right: 0;
  margin: 4px 0 0;
  padding: 4px;
  list-style: none;
  max-height: 180px;
  overflow-y: auto;
  border-radius: 8px;
  border: 1px solid ${(props) => props.theme.colors.separator};
  background: ${(props) => props.theme.colors.popoverBackground};
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.28);
`;

const Suggestion = styled.li<{ selected: boolean }>`
  display: flex;
  align-items: baseline;
  gap: 10px;
  padding: 5px 8px;
  border-radius: 5px;
  cursor: pointer;
  background: ${(props) => (props.selected ? "rgba(171, 40, 252, 0.14)" : "transparent")};

  &:hover {
    background: rgba(171, 40, 252, 0.1);
  }
`;

const SuggestionText = styled.span`
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 13px;
  color: ${(props) => props.theme.colors.text};
`;

const SuggestionHint = styled.span`
  font-size: 11px;
  color: ${(props) => props.theme.colors.secondaryText};
  margin-left: auto;
`;

/* Colours by role. `unknown-field` is the loud one on purpose: a mistyped
   field is the mistake people actually make, and it is otherwise invisible. */
const COLORS: Record<TokenKind, string> = {
  field: "#ab28fc",
  "unknown-field": "#d45769",
  op: "#8b8b93",
  value: "#2f9e6e",
  string: "#2f9e6e",
  logic: "#c77dff",
  paren: "#8b8b93",
  space: "inherit",
  error: "#d45769",
};

const Token = styled.span<{ kind: TokenKind }>`
  color: ${(props) => COLORS[props.kind]};
  ${(props) =>
    props.kind === "unknown-field" || props.kind === "error"
      ? "text-decoration: underline wavy; text-underline-offset: 3px;"
      : ""}
`;

export type RsqlEditorProps = {
  value: string;
  onChange: (value: string) => void;
  /** Called when the value settles — blur, or Enter with no suggestion open. */
  onCommit?: (value: string) => void;
  placeholder?: string;
};

const RsqlEditor: FC<RsqlEditorProps> = ({
  value,
  onChange,
  onCommit,
  placeholder,
}) => {
  const ref = useRef<HTMLTextAreaElement>(null);
  const [caret, setCaret] = useState(0);
  const [selected, setSelected] = useState(0);
  const [open, setOpen] = useState(false);

  const tokens = useMemo(() => tokenize(value, TRACK_FIELDS), [value]);
  const completion = useMemo(() => completionAt(value, caret), [value, caret]);
  const suggestions = useMemo(
    () => (open ? completion.suggestions.slice(0, 8) : []),
    [open, completion],
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
    [completion, onChange, value],
  );

  const sync = (node: HTMLTextAreaElement) => setCaret(node.selectionStart ?? 0);

  return (
    <Frame>
      <Editor>
        <Highlight aria-hidden>
          {tokens.map((token, i) => (
            <Token key={i} kind={token.kind}>
              {token.text}
            </Token>
          ))}
          {/* A trailing newline keeps the last line's height when the value
              ends in one, so the overlay does not shrink under the caret. */}
          {value.endsWith("\n") ? "\n" : ""}
        </Highlight>
        <Input
          ref={ref}
          rows={1}
          spellCheck={false}
          autoCapitalize="off"
          autoCorrect="off"
          value={value}
          placeholder={placeholder}
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
              setSelected((s) => (s - 1 + suggestions.length) % suggestions.length);
            } else if (e.key === "Enter" || e.key === "Tab") {
              e.preventDefault();
              accept(suggestions[selected].text);
            } else if (e.key === "Escape") {
              setOpen(false);
            }
          }}
        />
      </Editor>
      {suggestions.length > 0 && (
        <Suggestions>
          {suggestions.map((suggestion, i) => (
            <Suggestion
              key={suggestion.text}
              selected={i === selected}
              // `mouseDown` rather than `click`: the textarea's blur would
              // otherwise close the list before the click resolved.
              onMouseDown={(e) => {
                e.preventDefault();
                accept(suggestion.text);
              }}
            >
              <SuggestionText>{suggestion.text}</SuggestionText>
              {suggestion.hint && (
                <SuggestionHint>{suggestion.hint}</SuggestionHint>
              )}
            </Suggestion>
          ))}
        </Suggestions>
      )}
    </Frame>
  );
};

export default RsqlEditor;
