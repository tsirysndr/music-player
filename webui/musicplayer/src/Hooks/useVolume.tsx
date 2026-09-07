import { useAtom } from "jotai";
import { useCallback, useEffect } from "react";
import { fetcher } from "../Api/fetcher";
import { mutedAtom, volumeAtom, volumeLoadedAtom } from "../State";

/** How far one press of `+` or `-` moves the level. */
export const VOLUME_STEP = 0.05;

/**
 * The daemon's mixer, as shared state.
 *
 * In an atom rather than in the player bar because the shortcuts in `AppShell`
 * drive the same level the knob does, and two copies of it would disagree the
 * moment either one changed.
 *
 * Mute is separate from volume on purpose: the daemon keeps the level while
 * muted, so unmuting restores it rather than guessing. Setting a level unmutes
 * server-side, so the flag is cleared here to match.
 */
export const useVolume = () => {
  const [volume, setVolume] = useAtom(volumeAtom);
  const [muted, setMuted] = useAtom(mutedAtom);
  const [loaded, setLoaded] = useAtom(volumeLoadedAtom);

  // Once per session, not once per mount: both the player bar and the shell
  // call this hook and neither should trigger its own round trip.
  useEffect(() => {
    if (loaded) return;
    setLoaded(true);
    fetcher<any, any>(`query { getVolume getMute }`, {})()
      .then((data) => {
        setVolume((data.getVolume ?? 100) / 100);
        setMuted(!!data.getMute);
      })
      .catch(() => {});
  }, [loaded, setLoaded, setVolume, setMuted]);

  const change = useCallback(
    (next: number) => {
      const level = Math.min(1, Math.max(0, next));
      setVolume(level);
      setMuted(false);
      fetcher<any, any>(
        `mutation setVolume($volume: Int!) { setVolume(volume: $volume) }`,
        { volume: Math.round(level * 100) }
      )().catch(() => {});
    },
    [setVolume, setMuted]
  );

  /** Nudge by `delta` — what the `+` and `-` shortcuts do. */
  const adjust = useCallback(
    (delta: number) => change(volume + delta),
    [change, volume]
  );

  const toggleMute = useCallback(() => {
    const next = !muted;
    setMuted(next);
    fetcher<any, any>(`mutation setMute($mute: Boolean!) { setMute(mute: $mute) }`, {
      mute: next,
    })().catch(() => setMuted(!next));
  }, [muted, setMuted]);

  return { volume, muted, change, adjust, toggleMute };
};

export default useVolume;
