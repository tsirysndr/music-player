import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import MeterStrip from "./MeterStrip";

/** The colour class on each of the strip's segments, left to right. */
const segments = () =>
  Array.from(screen.getByTestId("meter-strip").children).map((segment) =>
    ["bg-meter-off", "bg-meter-low", "bg-meter-mid", "bg-meter-high"].find(
      (name) => segment.classList.contains(name)
    )
  );

describe("MeterStrip", () => {
  it("draws the requested number of segments", () => {
    render(<MeterStrip level={0} segments={8} />);
    expect(segments()).toHaveLength(8);
  });

  it("leaves every segment unlit at zero", () => {
    render(<MeterStrip level={0} />);
    expect(segments().every((name) => name === "bg-meter-off")).toBe(
      true
    );
  });

  /**
   * The top three segments are "high", the next three "mid", the rest "low" —
   * the banding is what makes a peak readable without a number.
   */
  it("bands the lit segments low → mid → high", () => {
    render(<MeterStrip level={1} />);
    const lit = segments();

    expect(lit.slice(0, 10).every((name) => name === "bg-meter-low")).toBe(true);
    expect(lit.slice(10, 13).every((name) => name === "bg-meter-mid")).toBe(
      true
    );
    expect(lit.slice(13).every((name) => name === "bg-meter-high")).toBe(true);
  });

  it("lights only up to the level", () => {
    render(<MeterStrip level={0.5} />);
    const lit = segments();

    expect(lit[7]).toBe("bg-meter-low");
    expect(lit[8]).toBe("bg-meter-off");
  });

  /** A NaN level would otherwise compare false everywhere and light nothing
      in a way that reads as a real zero — it should be an explicit zero. */
  it("treats a non-finite level as zero", () => {
    render(<MeterStrip level={NaN} />);
    expect(segments().every((name) => name === "bg-meter-off")).toBe(
      true
    );
  });
});
