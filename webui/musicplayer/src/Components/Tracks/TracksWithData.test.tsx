import { graphql, HttpResponse } from "msw";
import { describe, expect, it, vi } from "vitest";
import * as fixtures from "../../test/fixtures";
import { renderWithProviders, screen, waitFor } from "../../test/render";
import { server } from "../../test/server";
import TracksWithData from "./TracksWithData";

/**
 * Real tag data is untidy — this library has a title with a leading space —
 * and Testing Library compares against normalized text, so the expectation
 * has to be normalized too. The app renders the title as stored; trimming it
 * would be a scanner decision, not a display one.
 */
const title = (index: number) => fixtures.tracks[index].title.trim();

describe("TracksWithData", () => {
  it("renders the tracks the daemon returns", async () => {
    renderWithProviders(<TracksWithData />);
    expect(await screen.findByText(title(0))).toBeInTheDocument();
  });

  /**
   * The daemon stores durations as fractional seconds; the table shows mm:ss,
   * so the conversion has to happen somewhere and this is where.
   */
  it("formats the duration as mm:ss", async () => {
    server.use(
      graphql.query("GetTracks", () =>
        HttpResponse.json({
          data: {
            tracks: [{ ...fixtures.tracks[0], duration: 486.295 }],
          },
        })
      )
    );

    renderWithProviders(<TracksWithData />);
    expect(await screen.findByText("08:06")).toBeInTheDocument();
  });

  it("sends the filter to the daemon", async () => {
    const seen = vi.fn();
    server.use(
      graphql.query("GetTracks", ({ variables }) => {
        seen(variables);
        return HttpResponse.json({ data: { tracks: [] } });
      })
    );

    const { user } = renderWithProviders(<TracksWithData />);
    await waitFor(() => expect(seen).toHaveBeenCalled());

    const [box] = screen.getAllByRole("textbox", { name: "Filter tracks…" });
    await user.type(box, "loner");

    await waitFor(() =>
      expect(seen).toHaveBeenLastCalledWith(
        expect.objectContaining({ filter: "loner" })
      )
    );
  });

  it("queues a track from the row menu", async () => {
    const seen = vi.fn();
    server.use(
      graphql.mutation("PlayNext", ({ variables }) => {
        seen(variables);
        return HttpResponse.json({ data: { playNext: true } });
      })
    );

    const { user } = renderWithProviders(<TracksWithData />);
    await screen.findByText(title(0));

    const [menu] = screen.getAllByRole("button", { name: /More actions/ });
    await user.click(menu);
    await user.click(await screen.findByText("Play next"));

    await waitFor(() =>
      expect(seen).toHaveBeenCalledWith({ trackId: fixtures.tracks[0].id })
    );
  });

  it("likes a track from its row", async () => {
    const { user } = renderWithProviders(<TracksWithData />);
    await screen.findByText(title(0));

    const [like] = screen.getAllByRole("button", { name: "Add to liked" });
    await user.click(like);

    expect(
      screen.getAllByRole("button", { name: "Remove from liked" }).length
    ).toBeGreaterThan(0);
  });

  it("says so when the library is empty", async () => {
    server.use(
      graphql.query("GetTracks", () =>
        HttpResponse.json({ data: { tracks: [] } })
      )
    );

    renderWithProviders(<TracksWithData />);
    expect(
      await screen.findByText("No tracks in the library yet")
    ).toBeInTheDocument();
  });
});
