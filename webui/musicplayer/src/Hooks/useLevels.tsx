import { print } from "graphql";
import { useEffect, useRef, useState } from "react";
import { ON_LEVELS } from "../GraphQL/Playback/Subscription";
import { useGraphQLSubscription } from "./useGraphQLSubscription";

type LevelsPayload = {
  levels: {
    left: number;
    right: number;
    lowLeft: number;
    lowRight: number;
  };
};

/** What a meter needs: two 0..1 values, already scaled and eased. */
export type Levels = { left: number; right: number };

/**
 * Auto-gain: the loudest thing heard recently is the top of the meter.
 *
 * A fixed scale cannot work — bass level depends on the mix, the master and
 * the volume, so a factor chosen for one track leaves another at half height.
 * The reference jumps up instantly and falls slowly, so a quiet passage does
 * not immediately re-normalise into looking loud.
 */
const FLOOR = 0.02;
/** Per update at 20 Hz — about a 1.5 second half-life, which is release. */
const DECAY = 0.977;

/** Fast attack, slow release: real meter ballistics. */
const ease = (current: number, target: number) =>
  target > current
    ? current + (target - current) * 0.8
    : current + (target - current) * 0.15;

/**
 * The VU meter's levels, from the daemon's own measurement of the audio
 * leaving the output.
 *
 * The bass band, not the full-band RMS: a meter driven by everything sits near
 * the top on anything loud and reads as an ornament. This mirrors the Slint
 * desktop, so both clients move the same way on the same track.
 */
export function useLevels(playing: boolean): Levels {
  const [levels, setLevels] = useState<Levels>({ left: 0, right: 0 });
  const reference = useRef(FLOOR);
  const current = useRef<Levels>({ left: 0, right: 0 });

  useGraphQLSubscription<LevelsPayload>(print(ON_LEVELS), undefined, (data) => {
    const { lowLeft, lowRight } = data.levels;
    const peak = Math.max(lowLeft, lowRight);
    reference.current = Math.max(reference.current * DECAY, peak, FLOOR);

    const scale = (value: number) =>
      Math.min(1, Math.max(0, value / reference.current));
    const next = {
      left: ease(current.current.left, scale(lowLeft)),
      right: ease(current.current.right, scale(lowRight)),
    };
    current.current = next;
    setLevels(next);
  });

  // Stopped is a level too: a meter left holding its last reading looks stuck
  // rather than stopped.
  useEffect(() => {
    if (!playing) {
      current.current = { left: 0, right: 0 };
      setLevels({ left: 0, right: 0 });
    }
  }, [playing]);

  return levels;
}
