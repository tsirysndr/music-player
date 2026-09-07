import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import Knob from "./Knob";

describe("Knob", () => {
  it("reports its value and label to assistive tech", () => {
    render(
      <Knob norm={0.75} label="Volume" valueText="75%" onChange={vi.fn()} />
    );
    const knob = screen.getByRole("slider", { name: "Volume" });
    expect(knob).toHaveAttribute("aria-valuenow", "75");
    expect(screen.getByText("75%")).toBeInTheDocument();
  });

  it("turns up as the pointer is dragged up, and down as it is dragged down", () => {
    const onChange = vi.fn();
    render(<Knob norm={0.5} aria-label="Volume" onChange={onChange} />);
    const knob = screen.getByRole("slider");

    // 140px of travel is a full sweep, so 70px up is +0.5.
    fireEvent.pointerDown(knob, { clientY: 100 });
    fireEvent.pointerMove(window, { clientY: 30 });
    expect(onChange).toHaveBeenLastCalledWith(1);

    fireEvent.pointerMove(window, { clientY: 170 });
    expect(onChange).toHaveBeenLastCalledWith(0);
  });

  /**
   * The drag tracks total travel from where it started rather than summing
   * per-event deltas, so a wobble back and forth lands where it began.
   */
  it("tracks travel from the press, so a return lands back on the start value", () => {
    const onChange = vi.fn();
    render(<Knob norm={0.4} aria-label="Volume" onChange={onChange} />);
    const knob = screen.getByRole("slider");

    fireEvent.pointerDown(knob, { clientY: 200 });
    fireEvent.pointerMove(window, { clientY: 150 });
    fireEvent.pointerMove(window, { clientY: 260 });
    fireEvent.pointerMove(window, { clientY: 200 });

    expect(onChange).toHaveBeenLastCalledWith(0.4);
  });

  it("stops following the pointer once it is released", () => {
    const onChange = vi.fn();
    render(<Knob norm={0.5} aria-label="Volume" onChange={onChange} />);
    const knob = screen.getByRole("slider");

    fireEvent.pointerDown(knob, { clientY: 100 });
    fireEvent.pointerUp(window);
    onChange.mockClear();

    fireEvent.pointerMove(window, { clientY: 20 });
    expect(onChange).not.toHaveBeenCalled();
  });

  it("resets to the default on a double click", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(
      <Knob
        norm={0.1}
        defaultNorm={0.75}
        aria-label="Volume"
        onChange={onChange}
      />
    );

    await user.dblClick(screen.getByRole("slider"));
    expect(onChange).toHaveBeenCalledWith(0.75);
  });

  it("steps with the arrow keys", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(<Knob norm={0.5} aria-label="Volume" onChange={onChange} />);

    screen.getByRole("slider").focus();
    await user.keyboard("{ArrowUp}");
    expect(onChange).toHaveBeenLastCalledWith(0.52);

    await user.keyboard("{ArrowDown}");
    expect(onChange).toHaveBeenLastCalledWith(0.48);
  });

  it("turns on a scroll and swallows the event so the page does not move", () => {
    const onChange = vi.fn();
    render(<Knob norm={0.5} aria-label="Volume" onChange={onChange} />);
    const knob = screen.getByRole("slider");

    const wheel = new WheelEvent("wheel", {
      deltaY: -1,
      cancelable: true,
      bubbles: true,
    });
    knob.dispatchEvent(wheel);

    expect(onChange).toHaveBeenLastCalledWith(0.54);
    expect(wheel.defaultPrevented).toBe(true);
  });

  it("clamps at both ends", () => {
    const onChange = vi.fn();
    const { rerender } = render(
      <Knob norm={1} aria-label="Volume" onChange={onChange} />
    );
    const knob = screen.getByRole("slider");

    knob.dispatchEvent(
      new WheelEvent("wheel", { deltaY: -1, cancelable: true, bubbles: true })
    );
    expect(onChange).toHaveBeenLastCalledWith(1);

    rerender(<Knob norm={0} aria-label="Volume" onChange={onChange} />);
    knob.dispatchEvent(
      new WheelEvent("wheel", { deltaY: 1, cancelable: true, bubbles: true })
    );
    expect(onChange).toHaveBeenLastCalledWith(0);
  });
});
