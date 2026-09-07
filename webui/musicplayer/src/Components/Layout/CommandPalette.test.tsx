import { describe, expect, it, vi } from "vitest";
import { renderWithProviders, screen, within } from "../../test/render";
import { Icons } from "../UI";
import CommandPalette, { type PaletteEntry } from "./CommandPalette";

const entry = (
  key: string,
  kind: PaletteEntry["kind"],
  title: string,
  subtitle?: string,
  run = vi.fn()
): PaletteEntry => ({
  key,
  kind,
  title,
  subtitle,
  icon: Icons.music,
  run,
});

// Subtitles are all distinct so a title query cannot also match one — an
// artist row's title is exactly what the other rows carry as a subtitle.
const ENTRIES: PaletteEntry[] = [
  entry("track-1", "track", "End Of The Beginning", "from 13"),
  entry("album-1", "album", "13 (Deluxe Version)", "2013"),
  entry("artist-1", "artist", "Black Sabbath"),
  entry("playlist-1", "playlist", "Late night", "For the small hours"),
  entry("extension-1", "extension", "Lyrics Provider", "Fetches lyrics"),
];

const setup = (
  props: Partial<React.ComponentProps<typeof CommandPalette>> = {}
) =>
  renderWithProviders(
    <CommandPalette
      open
      query="sab"
      entries={ENTRIES}
      selected={0}
      onQueryChange={vi.fn()}
      onSelect={vi.fn()}
      onActivate={vi.fn()}
      onClose={vi.fn()}
      {...props}
    />
  );

describe("CommandPalette", () => {
  it("renders nothing while closed", () => {
    setup({ open: false });
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("opens with a focused search box", () => {
    setup({ query: "" });
    const box = screen.getByRole("textbox", { name: "Search" });
    expect(box).toBeInTheDocument();
    expect(box).toHaveFocus();
  });

  it("prompts before anything is typed", () => {
    setup({ query: "", entries: [] });
    expect(
      screen.getByText("Search your library, playlists and extensions")
    ).toBeInTheDocument();
  });

  it("lists every kind of result, labelled", () => {
    setup();
    for (const title of [
      "End Of The Beginning",
      "13 (Deluxe Version)",
      "Black Sabbath",
      "Late night",
      "Lyrics Provider",
    ]) {
      expect(screen.getByText(title)).toBeInTheDocument();
    }
    expect(screen.getByText("track")).toBeInTheDocument();
    expect(screen.getByText("extension")).toBeInTheDocument();
  });

  it("reports each keystroke", async () => {
    const onQueryChange = vi.fn();
    const { user } = setup({ query: "", onQueryChange });

    await user.type(screen.getByRole("textbox", { name: "Search" }), "b");
    expect(onQueryChange).toHaveBeenCalledWith("b");
  });

  it("says so when nothing matches", () => {
    setup({ entries: [] });
    expect(screen.getByText("Nothing matches “sab”")).toBeInTheDocument();
  });

  /** A slow search should not read as "no results". */
  it("says it is still searching", () => {
    setup({ entries: [], loading: true });
    expect(screen.getByText("Searching…")).toBeInTheDocument();
    expect(screen.queryByText(/Nothing matches/)).toBeNull();
  });

  it("moves the selection with the arrow keys, wrapping at both ends", async () => {
    const onSelect = vi.fn();
    const { user } = setup({ onSelect, selected: 0 });

    await user.keyboard("{ArrowDown}");
    expect(onSelect).toHaveBeenLastCalledWith(1);

    // From the first row, up wraps to the last.
    await user.keyboard("{ArrowUp}");
    expect(onSelect).toHaveBeenLastCalledWith(ENTRIES.length - 1);
  });

  it("wraps past the last row", async () => {
    const onSelect = vi.fn();
    const { user } = setup({ onSelect, selected: ENTRIES.length - 1 });

    await user.keyboard("{ArrowDown}");
    expect(onSelect).toHaveBeenLastCalledWith(0);
  });

  it("activates the highlighted row on Enter", async () => {
    const onActivate = vi.fn();
    const { user } = setup({ onActivate, selected: 2 });

    await user.keyboard("{Enter}");
    expect(onActivate).toHaveBeenCalledWith(ENTRIES[2]);
  });

  it("activates a row that is clicked", async () => {
    const onActivate = vi.fn();
    const { user } = setup({ onActivate });

    await user.click(screen.getByText("Late night"));
    expect(onActivate).toHaveBeenCalledWith(ENTRIES[3]);
  });

  /** Enter with nothing to activate must not throw. */
  it("does nothing on Enter with no results", async () => {
    const onActivate = vi.fn();
    const { user } = setup({ entries: [], onActivate });

    await user.keyboard("{Enter}");
    expect(onActivate).not.toHaveBeenCalled();
  });

  it("marks the highlighted row", () => {
    setup({ selected: 1 });
    const rows = screen.getAllByRole("listitem");
    expect(rows[1]).toHaveAttribute("data-selected", "true");
    expect(rows[0]).toHaveAttribute("data-selected", "false");
  });

  it("closes on Escape", async () => {
    const onClose = vi.fn();
    const { user } = setup({ onClose });

    await user.keyboard("{Escape}");
    expect(onClose).toHaveBeenCalled();
  });

  /**
   * An extension row carries a switch. It sits beside the row's button rather
   * than inside it — a button cannot contain another one — so flipping it must
   * not also activate the row.
   */
  it("renders a row action without swallowing it into the row button", async () => {
    const onActivate = vi.fn();
    const onToggle = vi.fn();
    const withAction: PaletteEntry = {
      ...ENTRIES[4],
      action: (
        <button type="button" onClick={onToggle}>
          toggle
        </button>
      ),
    };

    const { user } = setup({ entries: [withAction], onActivate });

    const row = screen.getByRole("listitem");
    await user.click(within(row).getByRole("button", { name: "toggle" }));

    expect(onToggle).toHaveBeenCalledTimes(1);
    expect(onActivate).not.toHaveBeenCalled();
  });
});
