/**
 * Tokenizing and completing RSQL, for the filter editor.
 *
 * Mirrors `rsql/src/editor.rs` on the Rust side so the web, the desktop and
 * the TUI colour and complete the same way. Both are deliberately forgiving: a
 * filter is invalid for most of the time it is being typed, so neither
 * function throws — they describe what is there, including the broken parts.
 */

import { FIELDS, fieldByName, opsFor, valuesFor } from "./fields";

export type TokenKind =
  | "field"
  | "unknown-field"
  | "op"
  | "value"
  | "string"
  | "logic"
  | "paren"
  | "space"
  | "error";

export type Token = {
  kind: TokenKind;
  text: string;
  start: number;
  end: number;
};

const NAMED_OPS = new Set([
  "eq", "ne", "neq", "gt", "ge", "gte", "lt", "le", "lte",
  "in", "out", "nin", "like", "contains",
  "null", "isnull", "notnull", "isnotnull",
]);

/** Operator runs that are already whole, rather than the start of a longer one. */
const COMPLETE_OPS = new Set(["==", "!=", ">=", "<=", ">", "<"]);

const isDelimiter = (c: string) =>
  /\s/.test(c) || ";,()=!<>".includes(c);

/**
 * Split `input` into coloured runs.
 *
 * The runs tile the input exactly — no gaps, no overlaps — because the
 * highlight overlay is laid out on top of the caret. A dropped or doubled
 * character would push the colours out of alignment.
 */
export function tokenize(input: string, knownFields: Set<string>): Token[] {
  const tokens: Token[] = [];
  let i = 0;

  const lastMeaningful = (): TokenKind | undefined => {
    for (let k = tokens.length - 1; k >= 0; k--) {
      if (tokens[k].kind !== "space") return tokens[k].kind;
    }
    return undefined;
  };

  /** Whether the innermost open paren followed an operator — a value list. */
  const opensValueList = (): boolean => {
    for (let k = tokens.length - 1; k >= 0; k--) {
      if (tokens[k].kind !== "paren") continue;
      for (let j = k - 1; j >= 0; j--) {
        if (tokens[j].kind === "space") continue;
        return tokens[j].kind === "op";
      }
      return false;
    }
    return false;
  };

  /** A bare word right after an operator is a value, not a field. */
  const expectsValue = (): boolean => {
    const last = lastMeaningful();
    if (last === "op") return true;
    if (last === "logic" || last === "value" || last === "string" || last === "paren") {
      return opensValueList();
    }
    return false;
  };

  const followsValue = (): boolean => {
    const last = lastMeaningful();
    return last === "value" || last === "string" || last === "paren";
  };

  while (i < input.length) {
    const start = i;
    const c = input[i];

    if (/\s/.test(c)) {
      while (i < input.length && /\s/.test(input[i])) i++;
      tokens.push({ kind: "space", text: input.slice(start, i), start, end: i });
      continue;
    }

    if (c === "(" || c === ")") {
      i++;
      tokens.push({ kind: "paren", text: c, start, end: i });
      continue;
    }

    if (c === ";" || c === ",") {
      i++;
      tokens.push({ kind: "logic", text: c, start, end: i });
      continue;
    }

    if (c === "'" || c === '"') {
      i++;
      let closed = false;
      while (i < input.length) {
        if (input[i] === "\\" && i + 1 < input.length) {
          i += 2;
          continue;
        }
        if (input[i] === c) {
          i++;
          closed = true;
          break;
        }
        i++;
      }
      // An unterminated string is the error, and it is the whole run.
      tokens.push({
        kind: closed ? "string" : "error",
        text: input.slice(start, i),
        start,
        end: i,
      });
      continue;
    }

    // `=name=` — the FIQL spelling. A lone `==` falls through below.
    if (c === "=" && input[i + 1] !== "=" && i + 1 < input.length) {
      let j = i + 1;
      while (j < input.length && input[j] !== "=") j++;
      if (j < input.length) {
        i = j + 1;
        const name = input.slice(start + 1, j).toLowerCase();
        tokens.push({
          kind: NAMED_OPS.has(name) ? "op" : "error",
          text: input.slice(start, i),
          start,
          end: i,
        });
      } else {
        // Still being typed: `=gt` with no closing `=`.
        i = input.length;
        tokens.push({ kind: "op", text: input.slice(start, i), start, end: i });
      }
      continue;
    }

    if ("=!<>".includes(c)) {
      while (i < input.length && "=!<>".includes(input[i])) i++;
      tokens.push({ kind: "op", text: input.slice(start, i), start, end: i });
      continue;
    }

    while (i < input.length && !isDelimiter(input[i])) i++;
    const word = input.slice(start, i);
    const lower = word.toLowerCase();
    let kind: TokenKind;
    if ((lower === "and" || lower === "or") && followsValue()) {
      kind = "logic";
    } else if (expectsValue()) {
      kind = "value";
    } else {
      kind = knownFields.has(lower) ? "field" : "unknown-field";
    }
    tokens.push({ kind, text: word, start, end: i });
  }

  return tokens;
}

export type Suggestion = {
  text: string;
  hint: string;
};

export type Completion = {
  /** What the caret is positioned to complete. */
  what: "field" | "op" | "value";
  /** The partial text already typed, which a suggestion replaces. */
  prefix: string;
  /** Range in the source a suggestion replaces. */
  from: number;
  to: number;
  /** The field this comparison is about, when known. */
  field?: string;
  suggestions: Suggestion[];
};

const byPrefix = (suggestions: Suggestion[], prefix: string): Suggestion[] => {
  if (!prefix) return suggestions;
  const lower = prefix.toLowerCase();
  return suggestions.filter((s) => s.text.toLowerCase().startsWith(lower));
};

/**
 * What the caret is in the middle of typing, and what to offer for it.
 */
export function completionAt(input: string, caret: number): Completion {
  const at = Math.max(0, Math.min(caret, input.length));
  const before = input.slice(0, at);

  // The word the caret is inside.
  let wordStart = at;
  while (wordStart > 0 && !isDelimiter(input[wordStart - 1]) && !"'\"".includes(input[wordStart - 1])) {
    wordStart--;
  }
  const word = before.slice(wordStart);

  // A run of operator characters ending at the caret. Whether it is finished
  // decides what comes next: `year>` wants a value, `genre=` could still
  // become `=like=`.
  const opRun = before.match(/[=!<>]+$/);
  if (opRun && !word) {
    const from = at - opRun[0].length;
    const fieldName = fieldBefore(input, from);
    if (COMPLETE_OPS.has(opRun[0])) {
      return {
        what: "value",
        prefix: "",
        from: at,
        to: at,
        field: fieldName,
        suggestions: valuesFor(fieldByName(fieldName ?? "")),
      };
    }
    return {
      what: "op",
      prefix: opRun[0],
      from,
      to: at,
      field: fieldName,
      suggestions: byPrefix(
        opsFor(fieldByName(fieldName ?? "")?.type).map(({ op, hint }) => ({
          text: op,
          hint,
        })),
        opRun[0],
      ),
    };
  }

  // Directly after an operator: a value.
  const afterOp = before.slice(0, wordStart).match(/(?:[=!<>]+|=[a-z]+=)\s*$/i);
  if (afterOp) {
    const fieldName = fieldBefore(input, wordStart);
    return {
      what: "value",
      prefix: word,
      from: wordStart,
      to: at,
      field: fieldName,
      suggestions: byPrefix(valuesFor(fieldByName(fieldName ?? "")), word),
    };
  }

  // Anywhere else a bare word can start: a field.
  return {
    what: "field",
    prefix: word,
    from: wordStart,
    to: at,
    suggestions: byPrefix(
      FIELDS.map((field) => ({ text: field.name, hint: field.label })),
      word,
    ),
  };
}

/** The field name of the comparison ending just before `offset`. */
function fieldBefore(input: string, offset: number): string | undefined {
  const before = input.slice(0, offset);
  const match = before.match(/([A-Za-z0-9_.-]+)\s*(?:[=!<>]+|=[a-z]+=)?\s*$/i);
  return match?.[1];
}
