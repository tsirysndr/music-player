import { QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook } from "@testing-library/react";
import { graphql, HttpResponse } from "msw";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { makeQueryClient } from "../test/render";
import { server } from "../test/server";
import { useLikes } from "./useLikes";

/**
 * Likes live in localStorage — the daemon has no favorites store — and every
 * toggle is also forwarded to Rocksky through the `likeTrack` mutation.
 */
const STORAGE_KEY = "liked-tracks";

const TRACK = "e5f9895f3560dce3181eff25ca349f43";
const OTHER = "f7a13b7b4d461c2682b0ed34c7157392";

const wrapper = ({ children }: { children: ReactNode }) => (
  <QueryClientProvider client={makeQueryClient()}>
    {children}
  </QueryClientProvider>
);

const setup = () => renderHook(() => useLikes(), { wrapper });

describe("useLikes", () => {
  beforeEach(() => localStorage.clear());

  it("starts with nothing liked", () => {
    const { result } = setup();
    expect(result.current.isLiked(TRACK)).toBe(false);
  });

  it("reads what a previous session saved", () => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify([TRACK]));
    const { result } = setup();
    expect(result.current.isLiked(TRACK)).toBe(true);
    expect(result.current.isLiked(OTHER)).toBe(false);
  });

  it("likes and unlikes, persisting each way", () => {
    const { result } = setup();

    act(() => result.current.toggleLike(TRACK));
    expect(result.current.isLiked(TRACK)).toBe(true);
    expect(JSON.parse(localStorage.getItem(STORAGE_KEY)!)).toContain(TRACK);

    act(() => result.current.toggleLike(TRACK));
    expect(result.current.isLiked(TRACK)).toBe(false);
    expect(JSON.parse(localStorage.getItem(STORAGE_KEY)!)).not.toContain(TRACK);
  });

  it("keeps the other likes when one is toggled", () => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify([TRACK, OTHER]));
    const { result } = setup();

    act(() => result.current.toggleLike(TRACK));
    expect(result.current.isLiked(TRACK)).toBe(false);
    expect(result.current.isLiked(OTHER)).toBe(true);
  });

  /** Two components reading the same store must not disagree about a like. */
  it("keeps every reader in step", () => {
    const first = setup();
    const second = setup();

    act(() => first.result.current.toggleLike(TRACK));

    expect(first.result.current.isLiked(TRACK)).toBe(true);
    expect(second.result.current.isLiked(TRACK)).toBe(true);
  });

  it("forwards the like to the daemon", async () => {
    const seen = vi.fn();
    server.use(
      graphql.mutation("LikeTrack", ({ variables }) => {
        seen(variables);
        return HttpResponse.json({ data: { likeTrack: true } });
      })
    );

    const { result } = setup();
    act(() => result.current.toggleLike(TRACK));

    await vi.waitFor(() =>
      expect(seen).toHaveBeenCalledWith({ id: TRACK, like: true })
    );
  });

  /**
   * localStorage is user-writable, and a malformed value must not take the
   * whole library page down with it.
   */
  it("recovers from a corrupt store rather than throwing", () => {
    localStorage.setItem(STORAGE_KEY, "{not json");
    const { result } = setup();

    expect(result.current.isLiked(TRACK)).toBe(false);
    act(() => result.current.toggleLike(TRACK));
    expect(result.current.isLiked(TRACK)).toBe(true);
  });

  /** A failed round trip must not roll back what the user just did locally. */
  it("keeps the local like when the mutation fails", async () => {
    server.use(
      graphql.mutation("LikeTrack", () =>
        HttpResponse.json({ errors: [{ message: "no rocksky token" }] })
      )
    );

    const { result } = setup();
    act(() => result.current.toggleLike(TRACK));

    expect(result.current.isLiked(TRACK)).toBe(true);
  });
});
