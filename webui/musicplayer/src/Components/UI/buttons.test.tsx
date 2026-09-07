import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import Button from "./Button";
import EmptyState from "./EmptyState";
import IconButton from "./IconButton";
import { Icons } from "./icons";
import LikeButton from "./LikeButton";
import PlayPauseButton from "./PlayPauseButton";

describe("Button", () => {
  it("renders its label and fires on click", async () => {
    const user = userEvent.setup();
    const onClick = vi.fn();
    render(<Button onClick={onClick}>New playlist</Button>);

    await user.click(screen.getByRole("button", { name: "New playlist" }));
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it("does not fire while disabled", async () => {
    const user = userEvent.setup();
    const onClick = vi.fn();
    render(
      <Button disabled onClick={onClick}>
        New playlist
      </Button>
    );

    await user.click(screen.getByRole("button"));
    expect(onClick).not.toHaveBeenCalled();
  });

  /** `type="button"`: a bare button inside a form would submit it. */
  it("defaults to a non-submitting button", () => {
    render(<Button>Cancel</Button>);
    expect(screen.getByRole("button")).toHaveAttribute("type", "button");
  });
});

describe("IconButton", () => {
  it("needs its accessible name from the caller", async () => {
    const user = userEvent.setup();
    const onClick = vi.fn();
    render(
      <IconButton icon={Icons.shuffle} aria-label="Shuffle" onClick={onClick} />
    );

    await user.click(screen.getByRole("button", { name: "Shuffle" }));
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it("draws the glyph in the accent colour when it is on", () => {
    const { container, rerender } = render(
      <IconButton icon={Icons.repeat} accented aria-label="Repeat" />
    );
    expect(container.querySelector("svg")).toHaveClass("text-accent");

    rerender(<IconButton icon={Icons.repeat} aria-label="Repeat" />);
    expect(container.querySelector("svg")).toHaveClass("text-dim");
  });
});

describe("PlayPauseButton", () => {
  it("names itself for what it will do, not for what is happening", () => {
    const { rerender } = render(<PlayPauseButton onClick={vi.fn()} />);
    expect(screen.getByRole("button", { name: "Play" })).toBeInTheDocument();

    rerender(<PlayPauseButton playing onClick={vi.fn()} />);
    expect(screen.getByRole("button", { name: "Pause" })).toBeInTheDocument();
  });

  it("fires on click", async () => {
    const user = userEvent.setup();
    const onClick = vi.fn();
    render(<PlayPauseButton onClick={onClick} />);

    await user.click(screen.getByRole("button"));
    expect(onClick).toHaveBeenCalledTimes(1);
  });
});

describe("LikeButton", () => {
  it("names itself for the action and reports its state", () => {
    const { rerender } = render(<LikeButton onClick={vi.fn()} />);
    const add = screen.getByRole("button", { name: "Add to liked" });
    expect(add).toHaveAttribute("aria-pressed", "false");

    rerender(<LikeButton liked onClick={vi.fn()} />);
    expect(
      screen.getByRole("button", { name: "Remove from liked" })
    ).toHaveAttribute("aria-pressed", "true");
  });

  it("fires on click", async () => {
    const user = userEvent.setup();
    const onClick = vi.fn();
    render(<LikeButton onClick={onClick} />);

    await user.click(screen.getByRole("button"));
    expect(onClick).toHaveBeenCalledTimes(1);
  });
});

describe("EmptyState", () => {
  it("shows a title, an optional hint and an optional action", () => {
    const { rerender } = render(<EmptyState title="No albums yet" />);
    expect(screen.getByText("No albums yet")).toBeInTheDocument();
    expect(screen.queryByRole("button")).toBeNull();

    rerender(
      <EmptyState
        icon={Icons.disc}
        title="No albums yet"
        hint="Scan a folder and they will show up here."
        action={<Button>Scan</Button>}
      />
    );
    expect(
      screen.getByText("Scan a folder and they will show up here.")
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Scan" })).toBeInTheDocument();
  });
});
