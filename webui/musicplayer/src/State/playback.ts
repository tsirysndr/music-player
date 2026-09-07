import { atom } from "jotai";

export type NowPlaying = {
  id?: string;
  title?: string;
  artist?: string;
  album?: string;
  albumId?: string;
  cover?: string;
  duration: number; // in milliseconds
  progress: number; // in milliseconds
  isPlaying?: boolean;
};

export const nowPlayingAtom = atom<NowPlaying>({
  duration: 0,
  progress: 0,
});

export const playbackIndexAtom = atom<number | undefined>(undefined);

export const playbackPositionAtom = atom(
  (get) => get(nowPlayingAtom).progress,
  (get, set, positionMs: number) =>
    set(nowPlayingAtom, { ...get(nowPlayingAtom), progress: positionMs })
);

/**
 * The daemon's mixer level, 0..1, and whether it is muted.
 *
 * Shared rather than owned by the player bar: the `+`/`-`/`m` shortcuts in
 * `AppShell` drive the same level the knob does.
 */
export const volumeAtom = atom(1);
export const mutedAtom = atom(false);
/** Guards the one-time read, so two mounts do not both fetch it. */
export const volumeLoadedAtom = atom(false);
