import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import Waveform, { positionAt } from "./Waveform";

describe("positionAt", () => {
  it("maps a click to where it is along the box", () => {
    expect(positionAt(100, 100, 200)).toBe(0);
    expect(positionAt(200, 100, 200)).toBe(0.5);
    expect(positionAt(300, 100, 200)).toBe(1);
  });

  // A drag can leave the element; clamping keeps the seek inside the track.
  it("clamps outside the box", () => {
    expect(positionAt(0, 100, 200)).toBe(0);
    expect(positionAt(9999, 100, 200)).toBe(1);
  });

  it("does not divide by a zero width", () => {
    expect(positionAt(50, 0, 0)).toBe(0);
  });
});

describe("Waveform", () => {
  const bars = Array.from({ length: 20 }, (_, i) => i * 12);

  // The layout must not jump when the analysis arrives a moment later.
  it("shows a flat line until the track has been analysed", () => {
    render(<Waveform bars={[]} progress={0} duration={0} />);
    expect(screen.getByTestId("waveform-empty")).toBeInTheDocument();
  });

  it("draws a bar per peak", () => {
    const { container } = render(
      <Waveform bars={bars} progress={0} duration={100} />
    );
    expect(container.querySelectorAll("[style*='height']")).toHaveLength(20);
  });

  it("seeks to where it was clicked", () => {
    const onSeek = vi.fn();
    render(<Waveform bars={bars} progress={0} duration={200} onSeek={onSeek} />);
    const waveform = screen.getByTestId("waveform");
    vi.spyOn(waveform, "getBoundingClientRect").mockReturnValue({
      left: 0,
      width: 100,
    } as DOMRect);

    fireEvent.click(waveform, { clientX: 25 });
    expect(onSeek).toHaveBeenCalledWith(50);
  });

  // Keyboard users get the same seek, which is why it is a slider.
  it("steps with the arrow keys", () => {
    const onSeek = vi.fn();
    render(
      <Waveform bars={bars} progress={0.5} duration={200} onSeek={onSeek} />
    );
    fireEvent.keyDown(screen.getByTestId("waveform"), { key: "ArrowRight" });
    expect(onSeek).toHaveBeenCalledWith(105);
  });

  it("is not a slider when it cannot seek", () => {
    render(<Waveform bars={bars} progress={0} duration={100} />);
    expect(screen.queryByRole("slider")).not.toBeInTheDocument();
  });
});
