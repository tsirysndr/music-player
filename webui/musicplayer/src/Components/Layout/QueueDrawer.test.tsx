import { graphql, HttpResponse } from "msw";
import { createStore, Provider as JotaiProvider } from "jotai";
import { QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";
import { MemoryRouter } from "react-router-dom";
import { describe, expect, it, vi } from "vitest";
import Providers from "../../Providers";
import { nowPlayingAtom, queueOpenAtom, type NowPlaying } from "../../State";
import { makeQueryClient } from "../../test/render";
import { server } from "../../test/server";
import QueueDrawer from "./QueueDrawer";

/** A track as the tracklist query returns it. */
const track = (id: string, title: string, artist: string, duration: number) => ({
  __typename: "Track",
  id,
  title,
  artist,
  duration,
  artists: [{ __typename: "Artist", id: "a1", name: artist }],
  album: {
    __typename: "Album",
    id: "3b16b0caed8f12736ae5debf7363fc79",
    title: "13 (Deluxe Version)",
    cover: null,
  },
});

const NEXT = [
  track("t1", "God Is Dead?", "Black Sabbath", 532.317),
  track("t2", "Loner", "Black Sabbath", 300.001),
];
const PREVIOUS = [track("t0", "End Of The Beginning", "Black Sabbath", 486.295)];

/**
 * The drawer renders nothing unless the queue is open, so it starts open.
 *
 * `nowPlayingAtom` is seeded here rather than through the GraphQL layer: it is
 * `AppStateSync` that fills it in the running app, and mounting the whole
 * shell to reach one card would test the shell instead of the drawer.
 */
const setup = (nowPlaying?: NowPlaying) => {
  const store = createStore();
  store.set(queueOpenAtom, true);
  if (nowPlaying) store.set(nowPlayingAtom, nowPlaying);

  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={makeQueryClient()}>
      <JotaiProvider store={store}>
        <Providers>
          <MemoryRouter>{children}</MemoryRouter>
        </Providers>
      </JotaiProvider>
    </QueryClientProvider>
  );

  return {
    store,
    user: userEvent.setup(),
    ...render(<QueueDrawer />, { wrapper }),
  };
};

const withQueue = (
  nextTracks = NEXT,
  previousTracks: typeof NEXT = PREVIOUS
) =>
  server.use(
    graphql.query("GetTracklist", () =>
      HttpResponse.json({
        data: {
          tracklistTracks: {
            __typename: "Tracklist",
            nextTracks,
            previousTracks,
          },
        },
      })
    )
  );

describe("QueueDrawer", () => {
  it("renders nothing while the queue is closed", () => {
    const store = createStore();
    store.set(queueOpenAtom, false);

    render(
      <QueryClientProvider client={makeQueryClient()}>
        <JotaiProvider store={store}>
          <Providers>
            <MemoryRouter>
              <QueueDrawer />
            </MemoryRouter>
          </Providers>
        </JotaiProvider>
      </QueryClientProvider>
    );

    expect(screen.queryByRole("heading", { name: "Queue" })).toBeNull();
  });

  it("lists what is up next", async () => {
    withQueue();
    setup();

    expect(await screen.findByText("God Is Dead?")).toBeInTheDocument();
    expect(screen.getByText("Loner")).toBeInTheDocument();
  });

  it("counts the whole queue, next and previous together", async () => {
    withQueue();
    setup();

    await screen.findByText("God Is Dead?");
    // Two up next plus one in history.
    expect(screen.getByText("3 tracks")).toBeInTheDocument();
  });

  it("formats each row's duration", async () => {
    withQueue();
    setup();
    expect(await screen.findByText("08:52")).toBeInTheDocument();
  });

  it("plays the row that is clicked, by its position", async () => {
    const seen = vi.fn();
    withQueue();
    server.use(
      graphql.mutation("PlayTrackAt", ({ variables }) => {
        seen(variables);
        return HttpResponse.json({ data: { playTrackAt: true } });
      })
    );

    const { user } = setup();
    await screen.findByText("Loner");

    await user.click(screen.getByText("Loner"));
    await waitFor(() => expect(seen).toHaveBeenCalledWith({ position: 1 }));
  });

  it("removes a row by its position", async () => {
    const seen = vi.fn();
    withQueue();
    server.use(
      graphql.mutation("RemoveTrackAt", ({ variables }) => {
        seen(variables);
        return HttpResponse.json({ data: { removeTrackAt: true } });
      })
    );

    const { user } = setup();
    await screen.findByText("Loner");

    await user.click(
      screen.getByRole("button", { name: "Remove Loner from the queue" })
    );
    await waitFor(() => expect(seen).toHaveBeenCalledWith({ position: 1 }));
  });

  it("switches to the history tab", async () => {
    withQueue();
    const { user } = setup();
    await screen.findByText("God Is Dead?");

    await user.click(screen.getByRole("button", { name: "History" }));

    expect(screen.getByText("End Of The Beginning")).toBeInTheDocument();
    expect(screen.queryByText("God Is Dead?")).toBeNull();
  });

  it("says so when nothing is up next", async () => {
    withQueue([], []);
    setup();
    expect(await screen.findByText("Nothing up next")).toBeInTheDocument();
  });

  it("says so when there is no history", async () => {
    withQueue([], []);
    const { user } = setup();
    await screen.findByText("Nothing up next");

    await user.click(screen.getByRole("button", { name: "History" }));
    expect(screen.getByText("No history yet")).toBeInTheDocument();
  });

  it("closes from its own button", async () => {
    withQueue();
    const { user, store } = setup();
    await screen.findByText("God Is Dead?");

    await user.click(screen.getByRole("button", { name: "Close queue" }));
    expect(store.get(queueOpenAtom)).toBe(false);
  });

  it("pins the now-playing card above the queue", async () => {
    withQueue();
    setup({
      id: "t0",
      title: "End Of The Beginning",
      artist: "Black Sabbath",
      duration: 486_295,
      progress: 0,
      isPlaying: true,
    });

    const label = await screen.findByText("NOW PLAYING");
    const card = within(label.parentElement!);
    expect(card.getByText("End Of The Beginning")).toBeInTheDocument();
    expect(card.getByText("Black Sabbath")).toBeInTheDocument();
  });

  it("has no now-playing card while nothing is playing", async () => {
    withQueue();
    setup();
    await screen.findByText("God Is Dead?");
    expect(screen.queryByText("NOW PLAYING")).toBeNull();
  });
});
