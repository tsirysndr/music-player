import { describe, expect, it, vi } from "vitest";
import { renderWithRouter, screen, within } from "../../test/render";
import AlbumCard, { type AlbumCardItem } from "./AlbumCard";

const ALBUM: AlbumCardItem = {
  id: "3b16b0caed8f12736ae5debf7363fc79",
  title: "13 (Deluxe Version)",
  artist: "Black Sabbath",
  year: 2013,
  cover: "/covers/3b16b0caed8f12736ae5debf7363fc79.jpg",
};

describe("AlbumCard", () => {
  it("shows the album, with the year after the artist", () => {
    renderWithRouter(<AlbumCard album={ALBUM} />);
    expect(screen.getByText("13 (Deluxe Version)")).toBeInTheDocument();
    expect(screen.getByText("Black Sabbath · 2013")).toBeInTheDocument();
  });

  it("shows the artist alone when the year is unknown", () => {
    renderWithRouter(<AlbumCard album={{ ...ALBUM, year: null }} />);
    expect(screen.getByText("Black Sabbath")).toBeInTheDocument();
  });

  it("links to the album", () => {
    renderWithRouter(<AlbumCard album={ALBUM} />);
    expect(screen.getByRole("link")).toHaveAttribute(
      "href",
      `/albums/${ALBUM.id}`
    );
  });

  it("renders the cover with the album as its alt text", () => {
    renderWithRouter(<AlbumCard album={ALBUM} />);
    expect(screen.getByAltText("13 (Deluxe Version)")).toHaveAttribute(
      "src",
      ALBUM.cover
    );
  });

  /** No cover: the skin's placeholder and disc glyph stand in, with no img. */
  it("falls back to the placeholder when there is no cover", () => {
    renderWithRouter(<AlbumCard album={{ ...ALBUM, cover: undefined }} />);
    expect(screen.queryByRole("img")).toBeNull();
  });

  it("plays from the overlay button", async () => {
    const onPlay = vi.fn();
    const { user } = renderWithRouter(
      <AlbumCard album={ALBUM} onPlay={onPlay} />
    );

    await user.click(
      screen.getByRole("button", { name: "Play 13 (Deluxe Version)" })
    );
    expect(onPlay).toHaveBeenCalledTimes(1);
  });

  it("offers the album actions in its menu", async () => {
    const onShufflePlay = vi.fn();
    const { user } = renderWithRouter(
      <AlbumCard
        album={ALBUM}
        onPlay={vi.fn()}
        onShufflePlay={onShufflePlay}
        onPlayNext={vi.fn()}
        onAddToQueue={vi.fn()}
        onLike={vi.fn()}
      />
    );

    await user.click(screen.getByRole("button", { name: /More actions/ }));
    const menu = await screen.findByRole("dialog");

    for (const label of [
      "Play",
      "Shuffle play",
      "Play next",
      "Add to queue",
      "Like album",
    ]) {
      expect(within(menu).getByText(label)).toBeInTheDocument();
    }

    await user.click(within(menu).getByText("Shuffle play"));
    expect(onShufflePlay).toHaveBeenCalledTimes(1);
  });

  it("shows no overlay buttons when it has no handlers", () => {
    renderWithRouter(<AlbumCard album={ALBUM} />);
    expect(screen.queryByRole("button")).toBeNull();
  });
});
