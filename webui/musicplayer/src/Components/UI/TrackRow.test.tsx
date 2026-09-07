import { describe, expect, it, vi } from "vitest";
import { renderWithRouter, screen, within } from "../../test/render";
import TrackRow, { TrackListHeader, type TrackRowItem } from "./TrackRow";

const TRACK: TrackRowItem = {
  id: "e5f9895f3560dce3181eff25ca349f43",
  title: "End Of The Beginning",
  artist: "Black Sabbath",
  artistId: "50fcd83c68216360292343903c891320",
  album: "13 (Deluxe Version)",
  albumId: "3b16b0caed8f12736ae5debf7363fc79",
  duration: "08:06",
  trackNumber: 1,
};

describe("TrackListHeader", () => {
  it("names the columns", () => {
    renderWithRouter(<TrackListHeader />);
    for (const label of ["#", "TITLE", "ARTIST", "ALBUM", "TIME"]) {
      expect(screen.getByText(label)).toBeInTheDocument();
    }
  });
});

describe("TrackRow", () => {
  it("shows the track", () => {
    renderWithRouter(
      <TrackRow track={TRACK} index={0} onPlay={vi.fn()} />
    );
    expect(screen.getByText("End Of The Beginning")).toBeInTheDocument();
    expect(screen.getByText("Black Sabbath")).toBeInTheDocument();
    expect(screen.getByText("13 (Deluxe Version)")).toBeInTheDocument();
    expect(screen.getByText("08:06")).toBeInTheDocument();
  });

  /**
   * A flat list numbers by position; an album view shows the metadata track
   * number, which is not the same thing once a disc is out of order.
   */
  it("numbers by position, or by track number when asked", () => {
    const { rerender } = renderWithRouter(
      <TrackRow track={{ ...TRACK, trackNumber: 9 }} index={2} onPlay={vi.fn()} />
    );
    expect(screen.getByText("3")).toBeInTheDocument();

    rerender(
      <TrackRow
        track={{ ...TRACK, trackNumber: 9 }}
        index={2}
        useTrackNumber
        onPlay={vi.fn()}
      />
    );
    expect(screen.getByText("9")).toBeInTheDocument();
  });

  it("falls back to the position when a track has no number", () => {
    renderWithRouter(
      <TrackRow
        track={{ ...TRACK, trackNumber: null }}
        index={4}
        useTrackNumber
        onPlay={vi.fn()}
      />
    );
    expect(screen.getByText("5")).toBeInTheDocument();
  });

  it("plays on click and on Enter", async () => {
    const onPlay = vi.fn();
    const { user } = renderWithRouter(
      <TrackRow track={TRACK} index={0} onPlay={onPlay} />
    );

    const row = screen.getByRole("button", {
      name: "Play End Of The Beginning",
    });
    await user.click(row);
    expect(onPlay).toHaveBeenCalledTimes(1);

    row.focus();
    await user.keyboard("{Enter}");
    expect(onPlay).toHaveBeenCalledTimes(2);
  });

  /** The artist and album cells are links, so clicking one must not also play. */
  it("navigates from the artist and album cells without playing", async () => {
    const onPlay = vi.fn();
    const { user } = renderWithRouter(
      <TrackRow track={TRACK} index={0} onPlay={onPlay} />
    );

    const artist = screen.getByRole("link", { name: "Black Sabbath" });
    expect(artist).toHaveAttribute("href", `/artists/${TRACK.artistId}`);
    await user.click(artist);
    expect(onPlay).not.toHaveBeenCalled();

    expect(
      screen.getByRole("link", { name: "13 (Deluxe Version)" })
    ).toHaveAttribute("href", `/albums/${TRACK.albumId}`);
  });

  it("renders plain text for an artist with no id", () => {
    renderWithRouter(
      <TrackRow
        track={{ ...TRACK, artistId: null, albumId: null }}
        index={0}
        onPlay={vi.fn()}
      />
    );
    expect(screen.queryByRole("link")).toBeNull();
    expect(screen.getByText("Black Sabbath")).toBeInTheDocument();
  });

  it("toggles the like without playing", async () => {
    const onPlay = vi.fn();
    const onLike = vi.fn();
    const { user } = renderWithRouter(
      <TrackRow track={TRACK} index={0} onPlay={onPlay} onLike={onLike} />
    );

    await user.click(screen.getByRole("button", { name: "Add to liked" }));
    expect(onLike).toHaveBeenCalledTimes(1);
    expect(onPlay).not.toHaveBeenCalled();
  });

  it("shows a liked track as liked", () => {
    renderWithRouter(
      <TrackRow
        track={{ ...TRACK, liked: true }}
        index={0}
        onPlay={vi.fn()}
        onLike={vi.fn()}
      />
    );
    expect(
      screen.getByRole("button", { name: "Remove from liked" })
    ).toHaveAttribute("aria-pressed", "true");
  });

  it("offers the queue actions in its menu", async () => {
    const onPlayNext = vi.fn();
    const onAddToQueue = vi.fn();
    const { user } = renderWithRouter(
      <TrackRow
        track={TRACK}
        index={0}
        onPlay={vi.fn()}
        onPlayNext={onPlayNext}
        onAddToQueue={onAddToQueue}
      />
    );

    await user.click(screen.getByRole("button", { name: /More actions/ }));
    await user.click(await screen.findByText("Play next"));
    expect(onPlayNext).toHaveBeenCalledTimes(1);
  });

  /**
   * The desktop offers the first three playlists inline and everything else
   * behind "Browse all playlists…", so a long list does not become the menu.
   */
  it("offers the first three playlists inline, then a browse entry", async () => {
    const onAddToPlaylist = vi.fn();
    const onBrowsePlaylists = vi.fn();
    const playlists = Array.from({ length: 5 }, (_, i) => ({
      id: `pl-${i}`,
      name: `Playlist ${i}`,
    }));

    const { user } = renderWithRouter(
      <TrackRow
        track={TRACK}
        index={0}
        playlists={playlists}
        onPlay={vi.fn()}
        onAddToPlaylist={onAddToPlaylist}
        onBrowsePlaylists={onBrowsePlaylists}
      />
    );

    await user.click(screen.getByRole("button", { name: /More actions/ }));
    const menu = await screen.findByRole("dialog");

    expect(within(menu).getByText("Playlist 0")).toBeInTheDocument();
    expect(within(menu).getByText("Playlist 2")).toBeInTheDocument();
    expect(within(menu).queryByText("Playlist 3")).toBeNull();
    expect(
      within(menu).getByText("Browse all playlists…")
    ).toBeInTheDocument();

    await user.click(within(menu).getByText("Playlist 1"));
    expect(onAddToPlaylist).toHaveBeenCalledWith("pl-1");
  });

  it("offers removal only when the list supports it", async () => {
    const onRemove = vi.fn();
    const { user } = renderWithRouter(
      <TrackRow
        track={TRACK}
        index={0}
        onPlay={vi.fn()}
        onRemove={onRemove}
      />
    );

    await user.click(screen.getByRole("button", { name: /More actions/ }));
    await user.click(await screen.findByText("Remove from playlist"));
    expect(onRemove).toHaveBeenCalledTimes(1);
  });

  it("has no menu at all when there is nothing to put in it", () => {
    renderWithRouter(
      <TrackRow
        track={{ ...TRACK, albumId: null }}
        index={0}
        onPlay={vi.fn()}
      />
    );
    expect(screen.queryByRole("button", { name: /More actions/ })).toBeNull();
  });
});
