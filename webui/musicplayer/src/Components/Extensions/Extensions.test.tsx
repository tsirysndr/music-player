import { describe, expect, it, vi } from "vitest";
import { renderWithProviders, screen } from "../../test/render";
import { extensions } from "../../test/handlers";
import Extensions, { type ExtensionItem } from "./Extensions";

const EXTENSIONS = extensions as unknown as ExtensionItem[];

const setup = (props: Partial<React.ComponentProps<typeof Extensions>> = {}) =>
  renderWithProviders(
    <Extensions
      extensions={EXTENSIONS}
      status="all"
      onStatusFilter={vi.fn()}
      onToggle={vi.fn()}
      onRescan={vi.fn()}
      {...props}
    />
  );

describe("Extensions", () => {
  it("lists every installed extension with its version", () => {
    setup();
    expect(screen.getByText("Lyrics Provider")).toBeInTheDocument();
    expect(screen.getByText("v1.2.0")).toBeInTheDocument();
    expect(screen.getByText("Mood Predicate")).toBeInTheDocument();
  });

  it("shows the id, so a user can find it in the CLI", () => {
    setup();
    expect(screen.getByText("fm.atradio.lyrics-provider")).toBeInTheDocument();
  });

  it("says whether each one is enabled", () => {
    setup();
    expect(
      screen.getByRole("switch", { name: "Enable Lyrics Provider" })
    ).toBeChecked();
    expect(
      screen.getByRole("switch", { name: "Enable Mood Predicate" })
    ).not.toBeChecked();
  });

  it("reports a switch flip as the state the user asked for", async () => {
    const onToggle = vi.fn();
    const { user } = setup({ onToggle });

    await user.click(
      screen.getByRole("switch", { name: "Enable Lyrics Provider" })
    );
    expect(onToggle).toHaveBeenCalledWith("fm.atradio.lyrics-provider", false);

    await user.click(
      screen.getByRole("switch", { name: "Enable Mood Predicate" })
    );
    expect(onToggle).toHaveBeenLastCalledWith("com.example.mood", true);
  });

  /** A second click while the first write is in flight would race it. */
  it("holds a switch while its own toggle is in flight", async () => {
    const onToggle = vi.fn();
    const { user } = setup({
      onToggle,
      pending: ["fm.atradio.lyrics-provider"],
    });

    await user.click(
      screen.getByRole("switch", { name: "Enable Lyrics Provider" })
    );
    expect(onToggle).not.toHaveBeenCalled();

    // Only that one is held; the others still work.
    await user.click(
      screen.getByRole("switch", { name: "Enable Mood Predicate" })
    );
    expect(onToggle).toHaveBeenCalledWith("com.example.mood", true);
  });

  it("rescans on request", async () => {
    const onRescan = vi.fn();
    const { user } = setup({ onRescan });

    await user.click(
      screen.getByRole("button", { name: "Rescan extensions" })
    );
    expect(onRescan).toHaveBeenCalledTimes(1);
  });

  describe("the quick filter", () => {
    it("marks the active chip", () => {
      setup({ status: "enabled" });
      expect(screen.getByRole("button", { name: "Enabled" })).toHaveAttribute(
        "aria-pressed",
        "true"
      );
      expect(screen.getByRole("button", { name: "All" })).toHaveAttribute(
        "aria-pressed",
        "false"
      );
    });

    it("reports the chip that was picked", async () => {
      const onStatusFilter = vi.fn();
      const { user } = setup({ onStatusFilter });

      await user.click(screen.getByRole("button", { name: "Disabled" }));
      expect(onStatusFilter).toHaveBeenCalledWith("disabled");
    });

    /** An empty chip is a different message from an empty library. */
    it("says which chip found nothing", () => {
      setup({ extensions: [], status: "disabled" });
      expect(screen.getByText("No disabled extensions")).toBeInTheDocument();
      expect(screen.queryByText(/extension init/)).toBeNull();
    });
  });

  it("shows what each one plugs into", () => {
    setup();
    expect(screen.getByText("metadata")).toBeInTheDocument();
    expect(screen.getByText("predicates")).toBeInTheDocument();
  });

  /**
   * A WebAssembly module is granted nothing by default, so what its manifest
   * asked for is the thing a user is on this page to check.
   */
  it("spells out what an extension may reach outside its sandbox", () => {
    setup();
    expect(screen.getByText("api.lyrics.example.com")).toBeInTheDocument();
    expect(screen.getByText("Reads your library")).toBeInTheDocument();
  });

  it("links to the project when it names one", () => {
    setup();
    const link = screen.getByRole("link", { name: "Website" });
    expect(link).toHaveAttribute("href", "https://example.com");
    // Opened in a new tab, so it cannot reach back into the app.
    expect(link).toHaveAttribute("rel", expect.stringContaining("noopener"));
  });

  it("shows a placeholder while loading", () => {
    const { container } = setup({ extensions: [], loading: true });
    expect(screen.queryByText("Lyrics Provider")).toBeNull();
    expect(container.querySelectorAll(".bg-skeleton").length).toBeGreaterThan(0);
  });

  it("explains how to get one when none is installed", () => {
    setup({ extensions: [] });
    expect(screen.getByText("No extensions installed")).toBeInTheDocument();
    expect(screen.getByText(/extension init/)).toBeInTheDocument();
  });


  it("surfaces a failure to load", () => {
    setup({ extensions: [], error: "Failed to fetch" });
    expect(screen.getByText("Unable to load extensions")).toBeInTheDocument();
    expect(screen.getByText("Failed to fetch")).toBeInTheDocument();
  });

});
