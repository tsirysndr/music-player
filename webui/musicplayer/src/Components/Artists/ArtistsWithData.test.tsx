import { graphql, HttpResponse } from "msw";
import { describe, expect, it, vi } from "vitest";
import * as fixtures from "../../test/fixtures";
import { renderWithProviders, screen, waitFor } from "../../test/render";
import { server } from "../../test/server";
import ArtistsWithData from "./ArtistsWithData";

describe("ArtistsWithData", () => {
  it("renders the artists the daemon returns", async () => {
    renderWithProviders(<ArtistsWithData />);
    expect(
      await screen.findByText(fixtures.artists[0].name)
    ).toBeInTheDocument();
  });

  it("links each row to the artist page", async () => {
    renderWithProviders(<ArtistsWithData />);
    const link = await screen.findByRole("link", {
      name: new RegExp(fixtures.artists[0].name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")),
    });
    expect(link).toHaveAttribute("href", `/artists/${fixtures.artists[0].id}`);
  });

  it("sends the filter to the daemon", async () => {
    const seen = vi.fn();
    server.use(
      graphql.query("GetArtists", ({ variables }) => {
        seen(variables);
        return HttpResponse.json({ data: { artists: [] } });
      })
    );

    const { user } = renderWithProviders(<ArtistsWithData />);
    await waitFor(() => expect(seen).toHaveBeenCalled());

    const [box] = screen.getAllByRole("textbox", { name: "Filter artists…" });
    await user.type(box, "sab");

    await waitFor(() =>
      expect(seen).toHaveBeenLastCalledWith(
        expect.objectContaining({ filter: "sab" })
      )
    );
  });

  it("plays an artist from their row", async () => {
    const seen = vi.fn();
    server.use(
      graphql.mutation("PlayArtistTracks", ({ variables }) => {
        seen(variables);
        return HttpResponse.json({ data: { playArtistTracks: true } });
      })
    );

    const { user } = renderWithProviders(<ArtistsWithData />);
    await screen.findByText(fixtures.artists[0].name);

    await user.click(
      screen.getByRole("button", { name: `Play ${fixtures.artists[0].name}` })
    );

    await waitFor(() =>
      expect(seen).toHaveBeenCalledWith(
        expect.objectContaining({ artistId: fixtures.artists[0].id })
      )
    );
  });

  it("says so when there are no artists", async () => {
    server.use(
      graphql.query("GetArtists", () =>
        HttpResponse.json({ data: { artists: [] } })
      )
    );

    renderWithProviders(<ArtistsWithData />);
    expect(await screen.findByText("No artists yet")).toBeInTheDocument();
  });
});
