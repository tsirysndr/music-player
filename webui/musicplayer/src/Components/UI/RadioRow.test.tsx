import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { Icons } from "./icons";
import { RadioCategoryTile, RadioRow, type StationItem } from "./RadioRow";

const STATION: StationItem = {
  id: "nightride-fm",
  name: "Nightride FM",
  subtitle: "synthwave · Sweden",
  source: "radio-browser",
  logo: "https://example.com/logo.png",
};

describe("RadioRow", () => {
  it("shows the station, its genre and where it came from", () => {
    render(<RadioRow station={STATION} onPlay={vi.fn()} />);
    expect(screen.getByText("Nightride FM")).toBeInTheDocument();
    expect(
      screen.getByText("synthwave · Sweden · radio-browser")
    ).toBeInTheDocument();
  });

  it("shows the logo when there is one", () => {
    render(<RadioRow station={STATION} onPlay={vi.fn()} />);
    expect(screen.getByAltText("Nightride FM")).toHaveAttribute(
      "src",
      STATION.logo
    );
  });

  /** No logo: the broadcast glyph on the skin's placeholder, and no img. */
  it("falls back to a glyph without one", () => {
    render(
      <RadioRow station={{ ...STATION, logo: null }} onPlay={vi.fn()} />
    );
    expect(screen.queryByRole("img")).toBeNull();
  });

  it("plays when the row is clicked", async () => {
    const user = userEvent.setup();
    const onPlay = vi.fn();
    render(<RadioRow station={STATION} onPlay={onPlay} />);

    await user.click(screen.getByText("Nightride FM"));
    expect(onPlay).toHaveBeenCalledTimes(1);
  });

  it("bookmarks without playing", async () => {
    const user = userEvent.setup();
    const onPlay = vi.fn();
    const onBookmark = vi.fn();
    render(
      <RadioRow station={STATION} onPlay={onPlay} onBookmark={onBookmark} />
    );

    await user.click(screen.getByRole("button", { name: "Add to liked" }));
    expect(onBookmark).toHaveBeenCalledTimes(1);
    expect(onPlay).not.toHaveBeenCalled();
  });

  it("shows a bookmarked station as bookmarked", () => {
    render(
      <RadioRow
        station={{ ...STATION, bookmarked: true }}
        onPlay={vi.fn()}
        onBookmark={vi.fn()}
      />
    );
    expect(
      screen.getByRole("button", { name: "Remove from liked" })
    ).toHaveAttribute("aria-pressed", "true");
  });

  it("has no bookmark button when the list cannot bookmark", () => {
    render(<RadioRow station={STATION} onPlay={vi.fn()} />);
    expect(screen.getAllByRole("button")).toHaveLength(1);
  });

  it("omits the separator when there is no source", () => {
    render(
      <RadioRow
        station={{ ...STATION, source: undefined }}
        onPlay={vi.fn()}
      />
    );
    expect(screen.getByText("synthwave · Sweden")).toBeInTheDocument();
  });
});

describe("RadioCategoryTile", () => {
  it("reports the label and the search term it stands for", async () => {
    const user = userEvent.setup();
    const onSelect = vi.fn();
    render(
      <RadioCategoryTile
        label="Synthwave"
        term="synthwave"
        icon={Icons.equalizer}
        color="#ff4fa3"
        onSelect={onSelect}
      />
    );

    await user.click(screen.getByRole("button", { name: /Synthwave/ }));
    expect(onSelect).toHaveBeenCalledWith("Synthwave", "synthwave");
  });

  /** Each category gets its own hue, which is what tells the grid apart. */
  it("tints its glyph with the category's colour", () => {
    const { container } = render(
      <RadioCategoryTile
        label="Jazz"
        term="jazz"
        icon={Icons.music}
        color="#f2c94c"
        onSelect={vi.fn()}
      />
    );
    expect(container.querySelector("svg")).toHaveStyle({ color: "#f2c94c" });
  });
});
