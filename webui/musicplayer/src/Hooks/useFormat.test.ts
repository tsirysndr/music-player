import { renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { useTimeFormat } from "./useFormat";

const format = () => renderHook(() => useTimeFormat()).result.current.formatTime;

describe("useTimeFormat", () => {
  it("formats milliseconds as mm:ss, zero-padded", () => {
    const formatTime = format();
    expect(formatTime(0)).toBe("00:00");
    expect(formatTime(9_000)).toBe("00:09");
    expect(formatTime(65_000)).toBe("01:05");
    expect(formatTime(600_000)).toBe("10:00");
  });

  /**
   * The scanner stores fractional seconds, so a duration multiplied up lands
   * on something like 486_295 rather than a whole minute.
   */
  it("rounds a fractional duration to the nearest second", () => {
    const formatTime = format();
    // 486.295s — "End Of The Beginning".
    expect(formatTime(486_295)).toBe("08:06");
  });

  /**
   * `toFixed` rounds 59.6s up to "60", which would otherwise render "07:60".
   */
  it("carries a rounded-up 60 seconds into the next minute", () => {
    const formatTime = format();
    expect(formatTime(59_600)).toBe("01:00");
    expect(formatTime(479_600)).toBe("08:00");
  });

  it("keeps counting past an hour rather than wrapping", () => {
    const formatTime = format();
    expect(formatTime(3_600_000)).toBe("60:00");
    expect(formatTime(5_400_000)).toBe("90:00");
  });
});
