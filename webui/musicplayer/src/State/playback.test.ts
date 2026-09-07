import { act, renderHook } from "@testing-library/react";
import { createStore, Provider, useAtom, useAtomValue, useSetAtom } from "jotai";
import { createElement, type ReactNode } from "react";
import { describe, expect, it } from "vitest";
import {
  nowPlayingAtom,
  playbackIndexAtom,
  playbackPositionAtom,
  type NowPlaying,
} from "./playback";

/**
 * Each test gets its own store. Jotai's default store is module-global, so
 * sharing it would let one test's playback state leak into the next.
 */
const withStore = () => {
  const store = createStore();
  const wrapper = ({ children }: { children: ReactNode }) =>
    createElement(Provider, { store }, children);
  return { store, wrapper };
};

const TRACK: NowPlaying = {
  id: "e5f9895f3560dce3181eff25ca349f43",
  title: "End Of The Beginning",
  artist: "Black Sabbath",
  album: "13 (Deluxe Version)",
  albumId: "3b16b0caed8f12736ae5debf7363fc79",
  duration: 486_295,
  progress: 0,
  isPlaying: true,
};

describe("nowPlayingAtom", () => {
  it("starts stopped, with nothing playing", () => {
    const { store } = withStore();
    expect(store.get(nowPlayingAtom)).toEqual({ duration: 0, progress: 0 });
  });

  it("holds the track the daemon reports", () => {
    const { store } = withStore();
    store.set(nowPlayingAtom, TRACK);
    expect(store.get(nowPlayingAtom).title).toBe("End Of The Beginning");
  });

  /**
   * `play()`/`pause()` update optimistically with a partial spread, so a
   * transport toggle must not drop the rest of the track.
   */
  it("keeps the rest of the track when only the play flag changes", () => {
    const { store } = withStore();
    store.set(nowPlayingAtom, TRACK);
    store.set(nowPlayingAtom, (previous) => ({ ...previous, isPlaying: false }));

    const current = store.get(nowPlayingAtom);
    expect(current.isPlaying).toBe(false);
    expect(current.title).toBe("End Of The Beginning");
    expect(current.duration).toBe(486_295);
  });

  it("notifies subscribers on change", () => {
    const { store, wrapper } = withStore();
    const { result } = renderHook(() => useAtomValue(nowPlayingAtom), {
      wrapper,
    });

    act(() => store.set(nowPlayingAtom, TRACK));
    expect(result.current.title).toBe("End Of The Beginning");
  });
});

describe("playbackPositionAtom", () => {
  it("reads the progress out of the now-playing track", () => {
    const { store } = withStore();
    store.set(nowPlayingAtom, { ...TRACK, progress: 42_000 });
    expect(store.get(playbackPositionAtom)).toBe(42_000);
  });

  /** Seeking writes the position back through this atom, not the whole track. */
  it("writes a new position without disturbing anything else", () => {
    const { store } = withStore();
    store.set(nowPlayingAtom, TRACK);
    store.set(playbackPositionAtom, 120_000);

    const current = store.get(nowPlayingAtom);
    expect(current.progress).toBe(120_000);
    expect(current.title).toBe("End Of The Beginning");
    expect(current.isPlaying).toBe(true);
  });

  it("stays in step when the track itself is replaced", () => {
    const { store } = withStore();
    store.set(nowPlayingAtom, { ...TRACK, progress: 90_000 });
    expect(store.get(playbackPositionAtom)).toBe(90_000);

    store.set(nowPlayingAtom, { ...TRACK, id: "next", progress: 0 });
    expect(store.get(playbackPositionAtom)).toBe(0);
  });

  it("re-renders a reader when the position is written", () => {
    const { wrapper } = withStore();
    const { result } = renderHook(
      () => {
        const [position, setPosition] = useAtom(playbackPositionAtom);
        return { position, setPosition };
      },
      { wrapper }
    );

    expect(result.current.position).toBe(0);
    act(() => result.current.setPosition(5_000));
    expect(result.current.position).toBe(5_000);
  });
});

describe("playbackIndexAtom", () => {
  it("is undefined until a queue position is known", () => {
    const { store } = withStore();
    expect(store.get(playbackIndexAtom)).toBeUndefined();
  });

  it("holds the queue position, including zero", () => {
    const { store, wrapper } = withStore();
    const { result } = renderHook(() => useSetAtom(playbackIndexAtom), {
      wrapper,
    });

    act(() => result.current(0));
    expect(store.get(playbackIndexAtom)).toBe(0);

    act(() => result.current(7));
    expect(store.get(playbackIndexAtom)).toBe(7);
  });
});
