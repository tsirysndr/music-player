import { act, renderHook, waitFor } from "@testing-library/react";
import { graphql, HttpResponse } from "msw";
import { createStore, Provider as JotaiProvider } from "jotai";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { mutedAtom, volumeAtom, volumeLoadedAtom } from "../State";
import { server } from "../test/server";
import { useVolume, VOLUME_STEP } from "./useVolume";

/**
 * The mixer calls are hand-written `fetcher` queries with no operation name,
 * so they are matched on the document text rather than by name.
 */
const mixer = (volume: number, mute: boolean, onWrite?: (body: any) => void) =>
  server.use(
    graphql.operation(async ({ query, variables }) => {
      if (query.includes("getVolume")) {
        return HttpResponse.json({ data: { getVolume: volume, getMute: mute } });
      }
      onWrite?.({ query, variables });
      if (query.includes("setVolume")) {
        return HttpResponse.json({ data: { setVolume: true } });
      }
      return HttpResponse.json({ data: { setMute: variables.mute } });
    })
  );

const setup = (seed?: (store: ReturnType<typeof createStore>) => void) => {
  const store = createStore();
  seed?.(store);
  const wrapper = ({ children }: { children: ReactNode }) => (
    <JotaiProvider store={store}>{children}</JotaiProvider>
  );
  return { store, ...renderHook(() => useVolume(), { wrapper }) };
};

/** Skips the one-time read so a seeded level survives the first render. */
const seeded = (volume: number, muted = false) =>
  setup((store) => {
    store.set(volumeLoadedAtom, true);
    store.set(volumeAtom, volume);
    store.set(mutedAtom, muted);
  });

describe("useVolume", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("reads the level and the mute flag once on mount", async () => {
    mixer(40, true);
    const { result } = setup();

    await waitFor(() => expect(result.current.volume).toBeCloseTo(0.4));
    expect(result.current.muted).toBe(true);
  });

  /**
   * Both the player bar and the shell call this hook. One fetch between them,
   * or the second mount races the first and the knob jumps.
   */
  it("does not fetch again once loaded", async () => {
    let reads = 0;
    server.use(
      graphql.operation(({ query }) => {
        if (query.includes("getVolume")) reads += 1;
        return HttpResponse.json({ data: { getVolume: 80, getMute: false } });
      })
    );

    const store = createStore();
    const wrapper = ({ children }: { children: ReactNode }) => (
      <JotaiProvider store={store}>{children}</JotaiProvider>
    );
    renderHook(() => useVolume(), { wrapper });
    renderHook(() => useVolume(), { wrapper });

    await waitFor(() => expect(store.get(volumeAtom)).toBeCloseTo(0.8));
    expect(reads).toBe(1);
  });

  it("sends the level as a percentage", async () => {
    const writes: any[] = [];
    mixer(100, false, (body) => writes.push(body));
    const { result } = seeded(0.5);

    act(() => result.current.change(0.63));

    await waitFor(() => expect(writes).toHaveLength(1));
    expect(writes[0].variables.volume).toBe(63);
  });

  it("adjusts by a step and clamps at both ends", async () => {
    mixer(100, false);
    const { result, store } = seeded(0.98);

    act(() => result.current.adjust(VOLUME_STEP));
    expect(store.get(volumeAtom)).toBe(1);

    act(() => result.current.adjust(-2));
    expect(store.get(volumeAtom)).toBe(0);
  });

  /** The daemon keeps the level while muted, so the knob must not move. */
  it("toggles mute without touching the level", async () => {
    mixer(100, false);
    const { result, store } = seeded(0.7);

    act(() => result.current.toggleMute());
    expect(store.get(mutedAtom)).toBe(true);
    expect(store.get(volumeAtom)).toBeCloseTo(0.7);

    act(() => result.current.toggleMute());
    await waitFor(() => expect(store.get(mutedAtom)).toBe(false));
  });

  it("unmutes when a level is set", async () => {
    mixer(100, false);
    const { result, store } = seeded(0.7, true);

    act(() => result.current.change(0.3));
    expect(store.get(mutedAtom)).toBe(false);
  });

  /** A failed toggle must not leave the button lying about the state. */
  it("rolls the mute flag back when the write fails", async () => {
    server.use(
      graphql.operation(({ query }) => {
        if (query.includes("getVolume")) {
          return HttpResponse.json({ data: { getVolume: 100, getMute: false } });
        }
        return HttpResponse.json({ errors: [{ message: "no mixer" }] });
      })
    );
    const { result, store } = seeded(0.7);

    act(() => result.current.toggleMute());
    expect(store.get(mutedAtom)).toBe(true);
    await waitFor(() => expect(store.get(mutedAtom)).toBe(false));
  });
});
