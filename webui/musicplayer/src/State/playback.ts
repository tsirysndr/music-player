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
