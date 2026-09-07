import { graphql, HttpResponse } from "msw";
import { describe, expect, it, vi } from "vitest";
import * as fixtures from "../../test/fixtures";
import { renderWithProviders, screen, waitFor } from "../../test/render";
import { server } from "../../test/server";
import AlbumDetailsWithData from "./AlbumDetailsWithData";

const render = () =>
  renderWithProviders(<AlbumDetailsWithData />, {
    route: `/albums/${fixtures.ALBUM_ID}`,
    path: "/albums/:id",
  });

/**
 * The heading, not just the text: the album's title also appears in the ALBUM
 * column of every row, so a plain text query matches a dozen elements.
 */
const heading = () =>
  screen.findByRole("heading", { name: fixtures.album.title });

describe("AlbumDetailsWithData", () => {
  it("asks for the album in the url", async () => {
    const seen = vi.fn();
    server.use(
      graphql.query("GetAlbum", ({ variables }) => {
        seen(variables);
        return HttpResponse.json({ data: { album: fixtures.album } });
      })
    );

    render();
    await waitFor(() =>
      expect(seen).toHaveBeenCalledWith({ id: fixtures.ALBUM_ID })
    );
  });

  it("shows the album and its tracklist", async () => {
    render();

    expect(await heading()).toBeInTheDocument();
    // The artist appears in the header and again on every row.
    expect(
      screen.getAllByText(fixtures.album.artist).length
    ).toBeGreaterThan(0);
    expect(
      screen.getByText(fixtures.album.tracks[0].title)
    ).toBeInTheDocument();
  });

  it("counts the tracks", async () => {
    render();
    await heading();
    expect(
      screen.getByText(new RegExp(`${fixtures.album.tracks.length} tracks`))
    ).toBeInTheDocument();
  });

  /**
   * An album view numbers by the metadata track number, not by row position —
   * the two differ as soon as a disc is out of order.
   */
  it("numbers the rows by their track number", async () => {
    render();
    await heading();

    for (const track of fixtures.album.tracks.slice(0, 3)) {
      expect(screen.getByText(String(track.trackNumber))).toBeInTheDocument();
    }
  });

  it("plays the album from the header", async () => {
    const seen = vi.fn();
    server.use(
      graphql.mutation("PlayAlbum", ({ variables }) => {
        seen(variables);
        return HttpResponse.json({ data: { playAlbum: true } });
      })
    );

    const { user } = render();
    await heading();

    await user.click(screen.getByRole("button", { name: "Play album" }));
    await waitFor(() =>
      expect(seen).toHaveBeenCalledWith(
        expect.objectContaining({ albumId: fixtures.ALBUM_ID, shuffle: false })
      )
    );
  });

  it("shuffles the album from the header", async () => {
    const seen = vi.fn();
    server.use(
      graphql.mutation("PlayAlbum", ({ variables }) => {
        seen(variables);
        return HttpResponse.json({ data: { playAlbum: true } });
      })
    );

    const { user } = render();
    await heading();

    await user.click(screen.getByRole("button", { name: "Shuffle album" }));
    await waitFor(() =>
      expect(seen).toHaveBeenCalledWith(
        expect.objectContaining({ shuffle: true })
      )
    );
  });

  /** Clicking a row plays the album *from that position*, not the track alone. */
  it("plays from the clicked row's position", async () => {
    const seen = vi.fn();
    server.use(
      graphql.mutation("PlayAlbum", ({ variables }) => {
        seen(variables);
        return HttpResponse.json({ data: { playAlbum: true } });
      })
    );

    const { user } = render();
    await heading();

    const third = fixtures.album.tracks[2];
    await user.click(
      screen.getByRole("button", { name: `Play ${third.title}` })
    );

    await waitFor(() =>
      expect(seen).toHaveBeenCalledWith(
        expect.objectContaining({ albumId: fixtures.ALBUM_ID, position: 2 })
      )
    );
  });

  it("shows a placeholder while it loads", () => {
    const { container } = render();
    expect(container.querySelectorAll(".bg-skeleton").length).toBeGreaterThan(0);
  });

  describe("discs", () => {
    /** A normal album must look exactly as it did — no stray header row. */
    it("does not label discs on a single-disc album", async () => {
      render();
      await heading();
      expect(screen.queryByText(/^DISC /)).toBeNull();
    });

    it("splits the table by disc when the album spans two", async () => {
      const half = Math.ceil(fixtures.album.tracks.length / 2);
      server.use(
        graphql.query("GetAlbum", () =>
          HttpResponse.json({
            data: {
              album: {
                ...fixtures.album,
                tracks: fixtures.album.tracks.map((track, index) => ({
                  ...track,
                  discNumber: index < half ? 1 : 2,
                })),
              },
            },
          })
        )
      );

      render();
      await heading();
      expect(await screen.findByText("DISC 1")).toBeInTheDocument();
      expect(screen.getByText("DISC 2")).toBeInTheDocument();
    });

    /**
     * The position sent to `playAlbum` indexes the whole album, not the disc —
     * the queue is the album, so disc 2's first track is not position 0.
     */
    it("keeps positions album-wide across discs", async () => {
      const seen = vi.fn();
      const half = Math.ceil(fixtures.album.tracks.length / 2);
      server.use(
        graphql.query("GetAlbum", () =>
          HttpResponse.json({
            data: {
              album: {
                ...fixtures.album,
                tracks: fixtures.album.tracks.map((track, index) => ({
                  ...track,
                  discNumber: index < half ? 1 : 2,
                })),
              },
            },
          })
        ),
        graphql.mutation("PlayAlbum", ({ variables }) => {
          seen(variables);
          return HttpResponse.json({ data: { playAlbum: true } });
        })
      );

      const { user } = render();
      await heading();

      const firstOfDiscTwo = fixtures.album.tracks[half];
      await user.click(
        screen.getByRole("button", { name: `Play ${firstOfDiscTwo.title}` })
      );

      await waitFor(() =>
        expect(seen).toHaveBeenCalledWith(
          expect.objectContaining({ position: half })
        )
      );
    });
  });

  describe("the album menu", () => {
    it("queues the whole album", async () => {
      const seen = vi.fn();
      server.use(
        graphql.mutation("AddTracks", ({ variables }) => {
          seen(variables);
          return HttpResponse.json({ data: { addTracks: true } });
        })
      );

      const { user } = render();
      await heading();

      await user.click(screen.getByRole("button", { name: "Album menu" }));
      await user.click(await screen.findByText("Add to queue"));

      await waitFor(() => expect(seen).toHaveBeenCalled());
      expect(seen.mock.calls[0][0].tracks).toHaveLength(
        fixtures.album.tracks.length
      );
      expect(seen.mock.calls[0][0].tracks[0]).toEqual(
        expect.objectContaining({
          id: fixtures.album.tracks[0].id,
          uri: fixtures.album.tracks[0].uri,
        })
      );
    });

    /** Liking an album is liking every track on it, as the desktop does. */
    it("likes every track on the album", async () => {
      const liked: string[] = [];
      server.use(
        graphql.mutation("LikeTrack", ({ variables }) => {
          liked.push(variables.id as string);
          return HttpResponse.json({ data: { likeTrack: true } });
        })
      );

      const { user } = render();
      await heading();

      await user.click(screen.getByRole("button", { name: "Album menu" }));
      await user.click(await screen.findByText("Like album"));

      await waitFor(() =>
        expect(liked).toHaveLength(fixtures.album.tracks.length)
      );
    });
  });
});
