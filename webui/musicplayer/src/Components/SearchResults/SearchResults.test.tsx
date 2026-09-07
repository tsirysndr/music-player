import { describe, expect, it, vi } from "vitest";
import { renderWithProviders, screen, within } from "../../test/render";
import SearchResults from "./SearchResults";

const TRACKS = [
  {
    id: "t1",
    title: "End Of The Beginning",
    artist: "Black Sabbath",
    artistId: "a1",
    album: "13 (Deluxe Version)",
    albumId: "al1",
    duration: "08:06",
  },
];

const ALBUMS = [
  {
    id: "al1",
    title: "13 (Deluxe Version)",
    artist: "Black Sabbath",
    year: 2013,
  },
];

const ARTISTS = [{ id: "a1", name: "Black Sabbath" }];

const setup = (
  props: Partial<React.ComponentProps<typeof SearchResults>> = {}
) =>
  renderWithProviders(
    <SearchResults
      query="sabbath"
      tracks={TRACKS}
      albums={ALBUMS}
      artists={ARTISTS}
      recentPlaylists={[]}
      onSearch={vi.fn()}
      onPlayTrack={vi.fn()}
      onPlayNext={vi.fn()}
      onToggleLike={vi.fn()}
      onAddTrackToPlaylist={vi.fn()}
      onPlayAlbum={vi.fn()}
      onPlayArtist={vi.fn()}
      {...props}
    />
  );

describe("SearchResults", () => {
  it("prompts before anything is typed", () => {
    setup({ query: "", tracks: [], albums: [], artists: [] });
    expect(screen.getByText("Search your library")).toBeInTheDocument();
    // No tabs until there is something to tab between.
    expect(screen.queryByRole("button", { name: /tracks/i })).toBeNull();
  });

  it("says so when a query matches nothing", () => {
    setup({ tracks: [], albums: [], artists: [] });
    expect(screen.getByText("Nothing matches “sabbath”")).toBeInTheDocument();
  });

  it("counts each kind of result on its tab", () => {
    setup();
    const tabs = screen.getByRole("button", { name: /tracks/i });
    expect(within(tabs).getByText("1")).toBeInTheDocument();
  });

  it("shows the tracks first", () => {
    setup();
    expect(screen.getByText("End Of The Beginning")).toBeInTheDocument();
  });

  it("switches to albums and artists", async () => {
    const { user } = setup();

    await user.click(screen.getByRole("button", { name: /albums/i }));
    expect(screen.getByText("Black Sabbath · 2013")).toBeInTheDocument();
    expect(screen.queryByText("End Of The Beginning")).toBeNull();

    await user.click(screen.getByRole("button", { name: /artists/i }));
    expect(
      screen.getByRole("link", { name: /Black Sabbath/ })
    ).toBeInTheDocument();
  });

  it("reports what is typed into the search box", async () => {
    const onSearch = vi.fn();
    const { user } = setup({ query: "", onSearch, tracks: [], albums: [], artists: [] });

    await user.type(
      screen.getByRole("textbox", { name: "Search your library" }),
      "b"
    );
    expect(onSearch).toHaveBeenCalledWith("b");
  });

  it("plays a track from its row", async () => {
    const onPlayTrack = vi.fn();
    const { user } = setup({ onPlayTrack });

    await user.click(
      screen.getByRole("button", { name: "Play End Of The Beginning" })
    );
    expect(onPlayTrack).toHaveBeenCalledWith("t1");
  });

  /** A tab with no results of its own is a narrower message than the page's. */
  it("says which tab is empty", async () => {
    const { user } = setup({ albums: [] });

    await user.click(screen.getByRole("button", { name: /albums/i }));
    expect(screen.getByText("No matching albums")).toBeInTheDocument();
  });
});
