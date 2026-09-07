import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import SlideBar from "./SlideBar";

/**
 * jsdom gives every element a zero-size rect, so a pointer-position test has
 * to supply the geometry itself. 200px wide starting at x=0 makes the
 * arithmetic in the assertions readable.
 */
const withWidth = (element: Element, width = 200, left = 0) => {
  vi.spyOn(element, "getBoundingClientRect").mockReturnValue({
    width,
    left,
    right: left + width,
    height: 5,
    top: 0,
    bottom: 5,
    x: left,
    y: 0,
    toJSON: () => ({}),
  });
};

describe("SlideBar", () => {
  it("reports its value to assistive tech", () => {
    render(<SlideBar progress={0.42} onChange={vi.fn()} />);
    expect(screen.getByRole("slider")).toHaveAttribute("aria-valuenow", "42");
  });

  it("clamps a progress outside 0..1 rather than overflowing the track", () => {
    const { rerender } = render(<SlideBar progress={2} onChange={vi.fn()} />);
    expect(screen.getByRole("slider")).toHaveAttribute("aria-valuenow", "100");

    rerender(<SlideBar progress={-1} onChange={vi.fn()} />);
    expect(screen.getByRole("slider")).toHaveAttribute("aria-valuenow", "0");
  });

  /** A NaN duration divides into NaN progress; the bar must not pass it on. */
  it("treats a non-finite progress as zero", () => {
    render(<SlideBar progress={NaN} onChange={vi.fn()} />);
    expect(screen.getByRole("slider")).toHaveAttribute("aria-valuenow", "0");
  });

  it("seeks to where the pointer went down", () => {
    const onChange = vi.fn();
    render(<SlideBar progress={0} onChange={onChange} />);
    const slider = screen.getByRole("slider");
    withWidth(slider.firstElementChild!);

    fireEvent.pointerDown(slider, { clientX: 50 });
    expect(onChange).toHaveBeenCalledWith(0.25);
  });

  it("keeps seeking while the pointer is dragged, and stops on release", () => {
    const onChange = vi.fn();
    render(<SlideBar progress={0} onChange={onChange} />);
    const slider = screen.getByRole("slider");
    withWidth(slider.firstElementChild!);

    fireEvent.pointerDown(slider, { clientX: 0 });
    fireEvent.pointerMove(window, { clientX: 100 });
    expect(onChange).toHaveBeenLastCalledWith(0.5);

    // Released outside the bar — the usual way a drag ends.
    fireEvent.pointerUp(window);
    onChange.mockClear();
    fireEvent.pointerMove(window, { clientX: 180 });
    expect(onChange).not.toHaveBeenCalled();
  });

  it("clamps a drag past either end of the track", () => {
    const onChange = vi.fn();
    render(<SlideBar progress={0.5} onChange={onChange} />);
    const slider = screen.getByRole("slider");
    withWidth(slider.firstElementChild!);

    fireEvent.pointerDown(slider, { clientX: 400 });
    expect(onChange).toHaveBeenLastCalledWith(1);

    fireEvent.pointerDown(slider, { clientX: -80 });
    expect(onChange).toHaveBeenLastCalledWith(0);
  });

  it("steps with the arrow keys, and further with shift held", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(<SlideBar progress={0.5} onChange={onChange} />);
    const slider = screen.getByRole("slider");

    slider.focus();
    await user.keyboard("{ArrowRight}");
    expect(onChange).toHaveBeenLastCalledWith(0.52);

    await user.keyboard("{ArrowLeft}");
    expect(onChange).toHaveBeenLastCalledWith(0.48);

    await user.keyboard("{Shift>}{ArrowRight}{/Shift}");
    expect(onChange).toHaveBeenLastCalledWith(0.6);
  });

  it("ignores pointer and keyboard input when disabled", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(<SlideBar progress={0.5} disabled onChange={onChange} />);
    const slider = screen.getByRole("slider");
    withWidth(slider.firstElementChild!);

    fireEvent.pointerDown(slider, { clientX: 50 });
    await user.keyboard("{ArrowRight}");

    expect(onChange).not.toHaveBeenCalled();
    expect(slider).toHaveAttribute("aria-disabled", "true");
  });

  /** A zero-width bar would otherwise divide by zero and emit NaN. */
  it("does not emit NaN when the track has no width yet", () => {
    const onChange = vi.fn();
    render(<SlideBar progress={0} onChange={onChange} />);
    const slider = screen.getByRole("slider");
    withWidth(slider.firstElementChild!, 0);

    fireEvent.pointerDown(slider, { clientX: 30 });
    expect(onChange).toHaveBeenCalledWith(0);
  });
});
