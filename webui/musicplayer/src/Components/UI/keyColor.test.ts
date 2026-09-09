import { describe, expect, it } from "vitest";
import { keyColorFor } from "./keyColor";

/** The hue out of an `hsl(...)` string, for comparing positions on the wheel. */
const hueOf = (color: string) => Number(color.match(/hsl\((\d+)/)![1]);

describe("keyColorFor", () => {
  it("gives every key a colour", () => {
    for (let n = 1; n <= 12; n++) {
      for (const mode of ["A", "B"]) {
        expect(keyColorFor(`${n}${mode}`)).not.toBeNull();
      }
    }
  });

  // Nothing is drawn for a track that has not been analysed.
  it("gives a non-key no colour", () => {
    for (const input of ["", "A", "13A", "0A", "8C", "8", "  ", null, undefined]) {
      expect(keyColorFor(input)).toBeNull();
    }
  });

  // The whole point: neighbours on the wheel mix, and must look alike.
  it("puts keys that mix next to each other on the hue wheel", () => {
    const step = Math.abs(hueOf(keyColorFor("8A")!) - hueOf(keyColorFor("9A")!));
    const across = Math.abs(hueOf(keyColorFor("8A")!) - hueOf(keyColorFor("2A")!));
    expect(step).toBeLessThan(across);
  });

  // Relative major and minor share a key signature, so they share a hue — but
  // must not be the same colour, or the two would be indistinguishable.
  it("shares a hue between relative keys without repeating the colour", () => {
    expect(hueOf(keyColorFor("8A")!)).toBe(hueOf(keyColorFor("8B")!));
    expect(keyColorFor("8A")).not.toBe(keyColorFor("8B"));
  });

  it("gives the twelve positions twelve distinct hues", () => {
    const hues = new Set(
      Array.from({ length: 12 }, (_, i) => hueOf(keyColorFor(`${i + 1}A`)!))
    );
    expect(hues.size).toBe(12);
  });

  it("accepts lower case and surrounding space", () => {
    expect(keyColorFor("8a")).toBe(keyColorFor("8A"));
    expect(keyColorFor(" 8A ")).toBe(keyColorFor("8A"));
  });
});
