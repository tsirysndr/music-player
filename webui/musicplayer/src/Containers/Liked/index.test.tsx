import { graphql, HttpResponse } from "msw";
import { describe, expect, it } from "vitest";
import { renderWithProviders, screen, waitFor } from "../../test/render";
import { server } from "../../test/server";
import LikedPage from ".";

/**
 * The liked page uses the hand-written `fetcher` rather than a codegen'd hook,
 * so its query has no operation name — MSW routes it by the field instead.
 */
const withLikedTracks = (tracks: unknown[]) =>
  server.use(
    graphql.operation(({ query }) => {
      if (!query.includes("likedTracks")) return;
      return HttpResponse.json({ data: { likedTracks: tracks } });
    })
  );

const TRACK = {
  id: "e5f9895f3560dce3181eff25ca349f43",
  title: "End Of The Beginning",
  artist: "Black Sabbath",
  duration: 486.295,
  artists: [{ id: "50fcd83c68216360292343903c891320" }],
  album: {
    id: "3b16b0caed8f12736ae5debf7363fc79",
    title: "13 (Deluxe Version)",
    cover: "3b16b0caed8f12736ae5debf7363fc79.jpg",
  },
};

describe("LikedPage", () => {
  it("lists the liked tracks", async () => {
    withLikedTracks([TRACK]);
    renderWithProviders(<LikedPage />);

    expect(
      await screen.findByText("End Of The Beginning")
    ).toBeInTheDocument();
    expect(screen.getByText("08:06")).toBeInTheDocument();
  });

  it("filters in place, without another request", async () => {
    withLikedTracks([
      TRACK,
      { ...TRACK, id: "other", title: "Loner", duration: 300 },
    ]);
    const { user } = renderWithProviders(<LikedPage />);
    await screen.findByText("Loner");

    const [box] = screen.getAllByRole("textbox", { name: "Filter liked…" });
    await user.type(box, "loner");

    expect(screen.getByText("Loner")).toBeInTheDocument();
    expect(screen.queryByText("End Of The Beginning")).toBeNull();
  });

  it("searches the album and artist too, not just the title", async () => {
    withLikedTracks([TRACK]);
    const { user } = renderWithProviders(<LikedPage />);
    await screen.findByText("End Of The Beginning");

    const [box] = screen.getAllByRole("textbox", { name: "Filter liked…" });
    await user.type(box, "sabbath");
    expect(screen.getByText("End Of The Beginning")).toBeInTheDocument();
  });

  /**
   * The likes are imported from the user's atproto repo, so an unlinked
   * account is an empty page rather than an error — and it should say why.
   */
  it("explains an empty page", async () => {
    withLikedTracks([]);
    renderWithProviders(<LikedPage />);

    expect(await screen.findByText("No liked song yet")).toBeInTheDocument();
    expect(screen.getByText(/Rocksky/)).toBeInTheDocument();
  });

  it("surfaces a failure rather than looking empty", async () => {
    server.use(
      graphql.operation(({ query }) => {
        if (!query.includes("likedTracks")) return;
        return HttpResponse.json({ errors: [{ message: "no atproto token" }] });
      })
    );

    renderWithProviders(<LikedPage />);
    expect(
      await screen.findByText("Unable to load liked songs")
    ).toBeInTheDocument();
    expect(screen.getByText("no atproto token")).toBeInTheDocument();
  });

  it("offers play and shuffle once there is something to play", async () => {
    withLikedTracks([TRACK]);
    renderWithProviders(<LikedPage />);
    await screen.findByText("End Of The Beginning");

    expect(
      screen.getByRole("button", { name: "Play liked tracks" })
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Shuffle liked tracks" })
    ).toBeInTheDocument();
  });

  it("hides the transport when there is nothing liked", async () => {
    withLikedTracks([]);
    renderWithProviders(<LikedPage />);
    await screen.findByText("No liked song yet");

    expect(
      screen.queryByRole("button", { name: "Play liked tracks" })
    ).toBeNull();
  });

  it("queues a track when one is played", async () => {
    withLikedTracks([TRACK]);
    let queued: unknown;
    server.use(
      graphql.mutation("PlayNext", ({ variables }) => {
        queued = variables;
        return HttpResponse.json({ data: { playNext: true } });
      })
    );

    const { user } = renderWithProviders(<LikedPage />);
    await screen.findByText("End Of The Beginning");

    await user.click(
      screen.getByRole("button", { name: "Play End Of The Beginning" })
    );
    await waitFor(() => expect(queued).toEqual({ trackId: TRACK.id }));
  });
});
