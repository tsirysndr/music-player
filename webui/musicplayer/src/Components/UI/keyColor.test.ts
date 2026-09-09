import { describe, expect, it } from "vitest";
import { keyColorFor, parseKey } from "./keyColor";

/** The hue out of an `hsl(...)` string, for comparing wheel positions. */
const hueOf = (color: string) => Number(color.match(/hsl\((\d+)/)![1]);

describe("parseKey", () => {
  // Both spellings name the same key, and tags in the wild use both.
  it("reads traditional and Camelot as the same key", () => {
    expect(parseKey("Fm")).toEqual(parseKey("4A"));
    expect(parseKey("D")).toEqual(parseKey("10B"));
    expect(parseKey("Am")).toEqual(parseKey("8A"));
  });

  it("understands accidentals in both spellings", () => {
    expect(parseKey("Abm")).toEqual(parseKey("G#m"));
    expect(parseKey("Bb")).toEqual(parseKey("A#"));
  });

  it("understands the long forms", () => {
    expect(parseKey("F minor")).toEqual(parseKey("Fm"));
    expect(parseKey("F major")).toEqual(parseKey("F"));
  });

  // A wrong key is worse than none, because it gets acted on.
  it("refuses anything that is not a key", () => {
    for (const input of ["", "  ", "Hm", "42", "F lydian", "13A", "0A", "8C", null, undefined]) {
      expect(parseKey(input)).toBeNull();
    }
  });
});

describe("keyColorFor", () => {
  it("colours every key and nothing else", () => {
    for (let n = 1; n <= 12; n++) {
      for (const mode of ["A", "B"]) {
        expect(keyColorFor(`${n}${mode}`)).not.toBeNull();
      }
    }
    expect(keyColorFor("Fm")).not.toBeNull();
    expect(keyColorFor("nonsense")).toBeNull();
  });

  // The whole point: keys that mix look alike, keys that clash do not.
  it("puts keys that mix next to each other on the hue wheel", () => {
    const step = Math.abs(hueOf(keyColorFor("Am")!) - hueOf(keyColorFor("Em")!));
    const across = Math.abs(hueOf(keyColorFor("Am")!) - hueOf(keyColorFor("D#m")!));
    expect(step).toBeLessThan(across);
  });

  // Relative keys share a key signature, so they share a hue — but must not be
  // the same colour, or they would be indistinguishable in a list.
  it("shares a hue between relative keys without repeating the colour", () => {
    expect(hueOf(keyColorFor("Am")!)).toBe(hueOf(keyColorFor("C")!));
    expect(keyColorFor("Am")).not.toBe(keyColorFor("C"));
  });

  it("gives the twelve positions twelve distinct hues", () => {
    const hues = new Set(
      Array.from({ length: 12 }, (_, i) => hueOf(keyColorFor(`${i + 1}A`)!))
    );
    expect(hues.size).toBe(12);
  });

  // The same key however it is written must be the same colour.
  it("does not depend on which notation was used", () => {
    expect(keyColorFor("Fm")).toBe(keyColorFor("4A"));
    expect(keyColorFor("D")).toBe(keyColorFor("10B"));
  });
});
