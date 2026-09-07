import { graphql, HttpResponse } from "msw";
import { describe, expect, it, vi } from "vitest";
import * as fixtures from "../../test/fixtures";
import { renderWithProviders, screen, waitFor } from "../../test/render";
import { server } from "../../test/server";
import ArtistDetailsWithData from "./ArtistDetailsWithData";

const render = () =>
  renderWithProviders(<ArtistDetailsWithData />, {
    route: `/artists/${fixtures.ARTIST_ID}`,
    path: "/artists/:id",
  });

/** The heading — the artist's name is also on every song row. */
const heading = () =>
  screen.findByRole("heading", { name: fixtures.artistDetail.name });

describe("ArtistDetailsWithData", () => {
  it("asks for the artist in the url", async () => {
    const seen = vi.fn();
    server.use(
      graphql.query("GetArtist", ({ variables }) => {
        seen(variables);
        return HttpResponse.json({ data: { artist: fixtures.artistDetail } });
      })
    );

    render();
    await waitFor(() =>
      expect(seen).toHaveBeenCalledWith({ id: fixtures.ARTIST_ID })
    );
  });

  it("shows the artist, their albums and their songs", async () => {
    render();

    expect(await heading()).toBeInTheDocument();
    expect(screen.getByText("ALBUMS")).toBeInTheDocument();
    expect(screen.getByText("SONGS")).toBeInTheDocument();
    expect(
      screen.getAllByText(fixtures.artistDetail.albums[0].title).length
    ).toBeGreaterThan(0);
  });

  it("counts the albums and songs", async () => {
    render();
    await heading();

    const albums = fixtures.artistDetail.albums.length;
    const songs = fixtures.artistDetail.songs.length;
    expect(
      screen.getByText(new RegExp(`${albums} albums? · ${songs} songs?`))
    ).toBeInTheDocument();
  });

  it("plays the artist from the header", async () => {
    const seen = vi.fn();
    server.use(
      graphql.mutation("PlayArtistTracks", ({ variables }) => {
        seen(variables);
        return HttpResponse.json({ data: { playArtistTracks: true } });
      })
    );

    const { user } = render();
    await heading();

    await user.click(
      screen.getByRole("button", { name: `Play ${fixtures.artistDetail.name}` })
    );
    await waitFor(() =>
      expect(seen).toHaveBeenCalledWith(
        expect.objectContaining({
          artistId: fixtures.ARTIST_ID,
          shuffle: false,
        })
      )
    );
  });

  /** An album card on this page plays that album, not the artist. */
  it("plays an album from its card", async () => {
    const seen = vi.fn();
    server.use(
      graphql.mutation("PlayAlbum", ({ variables }) => {
        seen(variables);
        return HttpResponse.json({ data: { playAlbum: true } });
      })
    );

    const { user } = render();
    await heading();

    const album = fixtures.artistDetail.albums[0];
    await user.click(screen.getByRole("button", { name: `Play ${album.title}` }));

    await waitFor(() =>
      expect(seen).toHaveBeenCalledWith(
        expect.objectContaining({ albumId: album.id })
      )
    );
  });

  it("hides the albums section for an artist with none in the library", async () => {
    server.use(
      graphql.query("GetArtist", () =>
        HttpResponse.json({
          data: { artist: { ...fixtures.artistDetail, albums: [] } },
        })
      )
    );

    render();
    await heading();
    expect(screen.queryByText("ALBUMS")).toBeNull();
    expect(screen.getByText("SONGS")).toBeInTheDocument();
  });
});
