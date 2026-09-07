import { describe, expect, it } from "vitest";
import { TRACK_FIELDS } from "./fields";
import { completionAt, tokenize, type TokenKind } from "./rsql";

/** The token kinds in order, with the whitespace runs dropped. */
const kinds = (input: string): TokenKind[] =>
  tokenize(input, TRACK_FIELDS)
    .filter((token) => token.kind !== "space")
    .map((token) => token.kind);

const texts = (input: string) =>
  tokenize(input, TRACK_FIELDS)
    .filter((token) => token.kind !== "space")
    .map((token) => token.text);

describe("tokenize", () => {
  /**
   * The highlight overlay is laid out on top of the caret, so the runs have to
   * tile the input exactly: a dropped or doubled character pushes the colours
   * out of alignment with the text the user is editing.
   */
  const tiles = (input: string) => {
    const tokens = tokenize(input, TRACK_FIELDS);
    expect(tokens.map((token) => token.text).join("")).toBe(input);
    let cursor = 0;
    for (const token of tokens) {
      expect(token.start).toBe(cursor);
      expect(token.end).toBe(cursor + token.text.length);
      cursor = token.end;
    }
  };

  it("returns nothing for an empty filter", () => {
    expect(tokenize("", TRACK_FIELDS)).toEqual([]);
  });

  it("tiles the input exactly, for every shape of filter", () => {
    for (const input of [
      "genre==rock",
      "genre==rock;year>2000",
      "  genre  ==  rock  ",
      "artist=='Black Sabbath'",
      "year=in=(2013,2014)",
      "nonsense==x",
      "genre==",
      "((year>2000);liked==true)",
      "!!!",
    ]) {
      tiles(input);
    }
  });

  it("colours a field, an operator and a value", () => {
    expect(kinds("genre==rock")).toEqual(["field", "op", "value"]);
    expect(texts("genre==rock")).toEqual(["genre", "==", "rock"]);
  });

  /** A mistyped field is the mistake people actually make. */
  it("marks a field that is not in the schema", () => {
    expect(kinds("mood==chill")).toEqual(["unknown-field", "op", "value"]);
  });

  it("colours the logical operators", () => {
    expect(kinds("genre==rock;year>2000")).toEqual([
      "field",
      "op",
      "value",
      "logic",
      "field",
      "op",
      "value",
    ]);
  });

  it("colours a quoted value as a string, quotes included", () => {
    expect(texts("artist=='Black Sabbath'")).toEqual([
      "artist",
      "==",
      "'Black Sabbath'",
    ]);
    expect(kinds("artist=='Black Sabbath'")).toContain("string");
  });

  it("handles a value list in parentheses", () => {
    expect(texts("year=in=(2013,2014)")).toEqual([
      "year",
      "=in=",
      "(",
      "2013",
      ",",
      "2014",
      ")",
    ]);
  });

  it("colours grouping parens", () => {
    expect(kinds("(year>2000)")).toEqual(["paren", "field", "op", "value", "paren"]);
  });

  /** A filter is invalid for most of the time it is being written. */
  it("describes a half-typed filter without throwing", () => {
    expect(() => tokenize("genre==", TRACK_FIELDS)).not.toThrow();
    expect(kinds("genre==")).toEqual(["field", "op"]);
    expect(kinds("genre")).toEqual(["field"]);
  });

  it("preserves whitespace as its own runs", () => {
    const tokens = tokenize("genre == rock", TRACK_FIELDS);
    expect(tokens.filter((token) => token.kind === "space")).toHaveLength(2);
  });
});

describe("completionAt", () => {
  it("suggests fields from a bare prefix", () => {
    const completion = completionAt("gen", 3);
    expect(completion.what).toBe("field");
    expect(completion.suggestions.map((s) => s.text)).toContain("genre");
  });

  it("suggests every field on an empty filter", () => {
    const completion = completionAt("", 0);
    expect(completion.suggestions.length).toBeGreaterThan(5);
  });

  it("suggests operators after a field", () => {
    const completion = completionAt("year=", 5);
    expect(completion.what).toBe("op");
    expect(completion.suggestions.map((s) => s.text)).toContain("==");
  });

  /** `>` is already whole, so what comes next is a value, not a longer op. */
  it("moves on to values once the operator is complete", () => {
    const completion = completionAt("year>", 5);
    expect(completion.what).toBe("value");
  });

  it("suggests the values of a boolean field", () => {
    const completion = completionAt("liked==", 7);
    expect(completion.what).toBe("value");
    expect(completion.suggestions.map((s) => s.text)).toEqual(
      expect.arrayContaining(["true", "false"])
    );
  });

  /** The replacement range is what the editor splices over. */
  it("reports the range the completion replaces", () => {
    const completion = completionAt("genre==rock;ye", 14);
    expect(completion.from).toBe(12);
    expect(completion.to).toBe(14);
    expect(completion.prefix).toBe("ye");
  });

  it("clamps a caret outside the input", () => {
    expect(() => completionAt("genre", 99)).not.toThrow();
    expect(() => completionAt("genre", -5)).not.toThrow();
    expect(completionAt("genre", 99).what).toBe("field");
  });

  it("completes the field after a logical operator, not the whole filter", () => {
    const completion = completionAt("genre==rock;ar", 14);
    expect(completion.what).toBe("field");
    expect(completion.suggestions.map((s) => s.text)).toContain("artist");
  });
});
