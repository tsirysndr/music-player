import { act, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { useLevels } from "./useLevels";

/** Captures the subscription callback so a test can push levels at it. */
let push: ((data: unknown) => void) | undefined;

vi.mock("./useGraphQLSubscription", () => ({
  useGraphQLSubscription: (
    _query: string,
    _variables: unknown,
    onData?: (data: unknown) => void
  ) => {
    push = onData;
    return { data: undefined };
  },
}));

const send = (lowLeft: number, lowRight = lowLeft) =>
  act(() => {
    push?.({ levels: { left: 1, right: 1, lowLeft, lowRight } });
  });

describe("useLevels", () => {
  /**
   * The whole point of the auto-gain: peaks reach the top whatever the
   * material's level. A fixed scale left quiet mixes at a fraction of the
   * height, which is what "the bars never pass 60%" was.
   */
  it("brings a peak to the top at any level", () => {
    for (const peak of [0.05, 0.2, 0.6, 1.0]) {
      const { result } = renderHook(() => useLevels(true));
      // Several buffers, because the attack is eased rather than instant.
      for (let i = 0; i < 40; i += 1) send(peak);
      expect(result.current.left).toBeGreaterThan(0.95);
    }
  });

  /** Between kicks it has to fall, or it just sits at the top. */
  it("reads lower on a quieter passage", () => {
    const { result } = renderHook(() => useLevels(true));
    for (let i = 0; i < 40; i += 1) send(0.5);
    for (let i = 0; i < 40; i += 1) send(0.05);
    expect(result.current.left).toBeLessThan(0.4);
  });

  /** A meter holding its last reading looks stuck rather than stopped. */
  it("falls to zero when playback stops", () => {
    const { result, rerender } = renderHook(
      ({ playing }) => useLevels(playing),
      { initialProps: { playing: true } }
    );
    for (let i = 0; i < 40; i += 1) send(0.8);
    expect(result.current.left).toBeGreaterThan(0.5);

    rerender({ playing: false });
    expect(result.current).toEqual({ left: 0, right: 0 });
  });

  it("keeps the channels independent", () => {
    const { result } = renderHook(() => useLevels(true));
    for (let i = 0; i < 40; i += 1) send(0.8, 0.1);
    expect(result.current.left).toBeGreaterThan(result.current.right);
  });
});
