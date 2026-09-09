import { describe, expect, it } from "vitest";
import { barTargets } from "./EqualizerBars";

describe("barTargets", () => {
  it("gives one height per bar, all within range", () => {
    const targets = barTargets(0.8, 0.8, 28, 0);
    expect(targets).toHaveLength(28);
    expect(targets.every((t) => t >= 0 && t <= 1)).toBe(true);
  });

  it("silence draws nothing", () => {
    expect(barTargets(0, 0, 12, 3).every((t) => t === 0)).toBe(true);
  });

  // The display is stereo: a sound only on the left must lift the left side.
  it("leans each side towards its own channel", () => {
    const targets = barTargets(1, 0, 20, 0);
    const leftEdge = targets[0];
    const rightEdge = targets[targets.length - 1];
    expect(leftEdge).toBeGreaterThan(rightEdge);
  });

  // Energy falls off with frequency, so a flat display would be wrong.
  it("tilts down towards the high end", () => {
    const targets = barTargets(1, 1, 20, 0);
    expect(targets[0]).toBeGreaterThan(targets[targets.length - 1]);
  });

  // Bass is sustained, treble is transient. Low bars should barely move
  // between frames while high ones flicker.
  it("moves the high bars more than the low ones", () => {
    const count = 20;
    const spread = (index: number) => {
      const seen = Array.from({ length: 40 }, (_, i) =>
        barTargets(0.7, 0.7, count, i * 0.05)[index]
      );
      return Math.max(...seen) - Math.min(...seen);
    };
    expect(spread(count - 1)).toBeGreaterThan(spread(0));
  });

  // NaN reaches here whenever a subscription drops a frame.
  it("treats a broken level as silence rather than drawing NaN", () => {
    expect(barTargets(NaN, NaN, 8, 1).every((t) => t === 0)).toBe(true);
  });

  it("survives a single bar", () => {
    expect(barTargets(0.5, 0.5, 1, 0)).toHaveLength(1);
  });
});
