import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import Toggle from "./Toggle";

describe("Toggle", () => {
  it("exposes itself as a switch with its checked state", () => {
    render(<Toggle checked label="Enable EQ" onChange={vi.fn()} />);
    expect(screen.getByRole("switch", { name: "Enable EQ" })).toBeChecked();
  });

  it("reports the opposite of its current state when clicked", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    const { rerender } = render(
      <Toggle checked={false} label="Enable EQ" onChange={onChange} />
    );

    await user.click(screen.getByRole("switch"));
    expect(onChange).toHaveBeenLastCalledWith(true);

    rerender(<Toggle checked label="Enable EQ" onChange={onChange} />);
    await user.click(screen.getByRole("switch"));
    expect(onChange).toHaveBeenLastCalledWith(false);
  });

  it("does not fire while disabled", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(<Toggle disabled label="Enable EQ" onChange={onChange} />);

    await user.click(screen.getByRole("switch"));
    expect(onChange).not.toHaveBeenCalled();
  });
});
