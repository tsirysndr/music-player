import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import Button from "./Button";
import Dialog from "./Dialog";

describe("Dialog", () => {
  it("renders nothing while closed", () => {
    render(
      <Dialog isOpen={false} onClose={vi.fn()} title="Create new playlist">
        <p>body</p>
      </Dialog>
    );
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("shows its title, body and footer when open", () => {
    render(
      <Dialog
        isOpen
        onClose={vi.fn()}
        title="Create new playlist"
        footer={<Button>Create</Button>}
      >
        <p>Give your playlist a title</p>
      </Dialog>
    );

    expect(screen.getByRole("dialog")).toBeInTheDocument();
    expect(screen.getByText("Create new playlist")).toBeInTheDocument();
    expect(screen.getByText("Give your playlist a title")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Create" })).toBeInTheDocument();
  });

  it("closes from the header button", async () => {
    const user = userEvent.setup();
    const onClose = vi.fn();
    render(
      <Dialog isOpen onClose={onClose} title="Create new playlist">
        <p>body</p>
      </Dialog>
    );

    await user.click(screen.getByRole("button", { name: "Close" }));
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("closes on Escape", async () => {
    const user = userEvent.setup();
    const onClose = vi.fn();
    render(
      <Dialog isOpen onClose={onClose} title="Create new playlist">
        <p>body</p>
      </Dialog>
    );

    await user.keyboard("{Escape}");
    await waitFor(() => expect(onClose).toHaveBeenCalled());
  });

  /**
   * A destructive confirmation opts out of backdrop dismissal — a stray click
   * outside a delete dialog should not be what decides it.
   */
  it("can refuse to dismiss on a backdrop click", async () => {
    const user = userEvent.setup();
    const onClose = vi.fn();
    render(
      <Dialog
        isOpen
        isDismissable={false}
        onClose={onClose}
        title="Delete playlist"
      >
        <p>body</p>
      </Dialog>
    );

    await user.click(document.body);
    expect(onClose).not.toHaveBeenCalled();
  });

  it("renders without a title", () => {
    render(
      <Dialog isOpen onClose={vi.fn()}>
        <p>Just a body</p>
      </Dialog>
    );
    expect(screen.getByText("Just a body")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Close" })).toBeNull();
  });
});
