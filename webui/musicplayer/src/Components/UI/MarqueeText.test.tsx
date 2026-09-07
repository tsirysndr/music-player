import { act, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import MarqueeText from "./MarqueeText";

/**
 * jsdom reports every element as zero-sized, so overflow has to be simulated:
 * `scrollWidth` is what the text measures, `clientWidth` what the box allows.
 */
function measure({ text, box }: { text: number; box: number }) {
  Object.defineProperty(HTMLSpanElement.prototype, "scrollWidth", {
    configurable: true,
    get: () => text,
  });
  Object.defineProperty(HTMLDivElement.prototype, "clientWidth", {
    configurable: true,
    get: () => box,
  });
}

describe("MarqueeText", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => {
    vi.useRealTimers();
    // @ts-expect-error — undo the prototype patches between tests.
    delete HTMLSpanElement.prototype.scrollWidth;
    // @ts-expect-error — same.
    delete HTMLDivElement.prototype.clientWidth;
  });

  it("renders the text", () => {
    measure({ text: 100, box: 400 });
    render(<MarqueeText text="Damaged Soul" />);
    expect(screen.getByText("Damaged Soul")).toBeInTheDocument();
  });

  /** Text that fits is left completely alone — only overlong titles move. */
  it("does not scroll text that fits", () => {
    measure({ text: 100, box: 400 });
    render(<MarqueeText text="Loner" />);
    const label = screen.getByText("Loner");

    expect(label).toHaveStyle({ transform: "translateX(0px)" });
    act(() => void vi.advanceTimersByTime(10_000));
    expect(label).toHaveStyle({ transform: "translateX(0px)" });
  });

  it("scrolls overlong text to the end, then back", () => {
    measure({ text: 400, box: 200 });
    render(<MarqueeText text="A very long ICY StreamTitle" speed={100} />);
    const label = screen.getByText("A very long ICY StreamTitle");

    expect(label).toHaveStyle({ transform: "translateX(0px)" });

    // 200px of overflow at 100px/s is 2s of travel, plus the 1.8s dwell.
    act(() => void vi.advanceTimersByTime(3_900));
    expect(label).toHaveStyle({ transform: "translateX(-200px)" });

    act(() => void vi.advanceTimersByTime(3_900));
    expect(label).toHaveStyle({ transform: "translateX(0px)" });
  });

  /** A new song restarts the run rather than continuing mid-scroll. */
  it("restarts from the beginning when the text changes", () => {
    measure({ text: 400, box: 200 });
    const { rerender } = render(<MarqueeText text="First title" speed={100} />);

    act(() => void vi.advanceTimersByTime(3_900));
    expect(screen.getByText("First title")).toHaveStyle({
      transform: "translateX(-200px)",
    });

    rerender(<MarqueeText text="Second title" speed={100} />);
    expect(screen.getByText("Second title")).toHaveStyle({
      transform: "translateX(0px)",
    });
  });
});
