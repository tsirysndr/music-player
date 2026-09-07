import { QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook } from "@testing-library/react";
import { graphql, HttpResponse } from "msw";
import { createStore, Provider as JotaiProvider } from "jotai";
import type { ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";
import { nowPlayingAtom, type NowPlaying } from "../State";
import { makeQueryClient } from "../test/render";
import { server } from "../test/server";
import { usePlayTrack } from "./usePlayTrack";

const TRACK = "e5f9895f3560dce3181eff25ca349f43";

const setup = (nowPlaying?: NowPlaying) => {
  const store = createStore();
  if (nowPlaying) store.set(nowPlayingAtom, nowPlaying);

  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={makeQueryClient()}>
      <JotaiProvider store={store}>{children}</JotaiProvider>
    </QueryClientProvider>
  );

  return renderHook(() => usePlayTrack(), { wrapper });
};

/** Record the order the mutations arrive in. */
const record = () => {
  const calls: string[] = [];
  server.use(
    graphql.mutation("PlayNext", () => {
      calls.push("playNext");
      return HttpResponse.json({ data: { playNext: true } });
    }),
    graphql.mutation("Next", () => {
      calls.push("next");
      return HttpResponse.json({ data: { next: true } });
    }),
    graphql.mutation("Play", () => {
      calls.push("play");
      return HttpResponse.json({ data: { play: true } });
    })
  );
  return calls;
};

describe("usePlayTrack", () => {
  /**
   * The daemon has no "play this track" mutation: the tracklist API can only
   * queue a track or jump to a position it already holds. So playing one is
   * queue-then-skip, and the order matters — skipping first would land on
   * whatever was already next.
   */
  it("queues the track and then skips onto it", async () => {
    const calls = record();
    const { result } = setup({
      id: "something-else",
      title: "End Of The Beginning",
      duration: 0,
      progress: 0,
      isPlaying: true,
    });

    await act(() => result.current(TRACK));
    expect(calls).toEqual(["playNext", "next"]);
  });

  /**
   * With nothing playing there is no current track to skip past, so the queued
   * track is already the one that starts — skipping would drop it.
   */
  it("starts playback instead of skipping when nothing is playing", async () => {
    const calls = record();
    const { result } = setup();

    await act(() => result.current(TRACK));
    expect(calls).toEqual(["playNext", "play"]);
  });

  it("queues the track it was given", async () => {
    const seen = vi.fn();
    server.use(
      graphql.mutation("PlayNext", ({ variables }) => {
        seen(variables);
        return HttpResponse.json({ data: { playNext: true } });
      })
    );

    const { result } = setup();
    await act(() => result.current(TRACK));

    expect(seen).toHaveBeenCalledWith({ trackId: TRACK });
  });
});
