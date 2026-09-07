import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import FilterBox from "./FilterBox";

describe("FilterBox", () => {
  it("labels itself from its placeholder when nothing else is given", () => {
    render(
      <FilterBox value="" placeholder="Filter albums…" onChange={vi.fn()} />
    );
    expect(
      screen.getByRole("textbox", { name: "Filter albums…" })
    ).toBeInTheDocument();
  });

  it("prefers an explicit label", () => {
    render(
      <FilterBox
        value=""
        placeholder="Search…"
        aria-label="Search your library"
        onChange={vi.fn()}
      />
    );
    expect(
      screen.getByRole("textbox", { name: "Search your library" })
    ).toBeInTheDocument();
  });

  it("reports each keystroke", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(<FilterBox value="" onChange={onChange} />);

    await user.type(screen.getByRole("textbox"), "sab");
    // Controlled and never re-rendered here, so every keystroke reports one
    // character rather than an accumulating string.
    expect(onChange).toHaveBeenCalledTimes(3);
    expect(onChange).toHaveBeenLastCalledWith("b");
  });

  /** The clear button only exists once there is something to clear. */
  it("shows a clear button only when it has a value", () => {
    const { rerender } = render(<FilterBox value="" onChange={vi.fn()} />);
    expect(screen.queryByRole("button", { name: "Clear filter" })).toBeNull();

    rerender(<FilterBox value="sabbath" onChange={vi.fn()} />);
    expect(
      screen.getByRole("button", { name: "Clear filter" })
    ).toBeInTheDocument();
  });

  it("clears to an empty string", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(<FilterBox value="sabbath" onChange={onChange} />);

    await user.click(screen.getByRole("button", { name: "Clear filter" }));
    expect(onChange).toHaveBeenCalledWith("");
  });

  it("shows the value it is given", () => {
    render(<FilterBox value="black" onChange={vi.fn()} />);
    expect(screen.getByRole("textbox")).toHaveValue("black");
  });
});
