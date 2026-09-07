import { graphql, HttpResponse } from "msw";
import { describe, expect, it, vi } from "vitest";
import * as fixtures from "../../test/fixtures";
import { renderWithProviders, screen, waitFor } from "../../test/render";
import { server } from "../../test/server";
import AlbumsWithData from "./AlbumsWithData";

describe("AlbumsWithData", () => {
  it("renders the albums the daemon returns", async () => {
    renderWithProviders(<AlbumsWithData />);

    expect(
      await screen.findByText(fixtures.albums[0].title)
    ).toBeInTheDocument();
    expect(screen.getByText(fixtures.albums[1].title)).toBeInTheDocument();
  });

  it("resolves each cover against the daemon's /covers/ route", async () => {
    renderWithProviders(<AlbumsWithData />);

    const cover = await screen.findByAltText(fixtures.albums[0].title);
    expect(cover).toHaveAttribute(
      "src",
      `/covers/${fixtures.albums[0].cover}`
    );
  });

  /**
   * A remote source can return the same album on more than one page, and two
   * cards for one album is the visible symptom.
   */
  it("shows one card per album even when a page repeats one", async () => {
    const duplicated = [fixtures.albums[0], fixtures.albums[0]];
    server.use(
      graphql.query("GetAlbums", () =>
        HttpResponse.json({ data: { albums: duplicated } })
      )
    );

    renderWithProviders(<AlbumsWithData />);

    await screen.findByText(fixtures.albums[0].title);
    expect(screen.getAllByText(fixtures.albums[0].title)).toHaveLength(1);
  });

  it("sends the filter to the daemon rather than filtering locally", async () => {
    const seen = vi.fn();
    server.use(
      graphql.query("GetAlbums", ({ variables }) => {
        seen(variables);
        return HttpResponse.json({ data: { albums: [] } });
      })
    );

    const { user } = renderWithProviders(<AlbumsWithData />);
    await waitFor(() => expect(seen).toHaveBeenCalled());

    const [box] = screen.getAllByRole("textbox", { name: "Filter albums…" });
    await user.type(box, "sab");

    await waitFor(() =>
      expect(seen).toHaveBeenLastCalledWith(
        expect.objectContaining({ filter: "sab" })
      )
    );
  });

  it("plays an album from its card", async () => {
    const seen = vi.fn();
    server.use(
      graphql.mutation("PlayAlbum", ({ variables }) => {
        seen(variables);
        return HttpResponse.json({ data: { playAlbum: true } });
      })
    );

    const { user } = renderWithProviders(<AlbumsWithData />);
    await screen.findByText(fixtures.albums[0].title);

    await user.click(
      screen.getByRole("button", { name: `Play ${fixtures.albums[0].title}` })
    );

    await waitFor(() =>
      expect(seen).toHaveBeenCalledWith(
        expect.objectContaining({
          albumId: fixtures.albums[0].id,
          shuffle: false,
        })
      )
    );
  });

  it("says so when the library is empty", async () => {
    server.use(
      graphql.query("GetAlbums", () =>
        HttpResponse.json({ data: { albums: [] } })
      )
    );

    renderWithProviders(<AlbumsWithData />);
    expect(await screen.findByText("No albums yet")).toBeInTheDocument();
  });
});
