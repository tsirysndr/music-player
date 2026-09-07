import { useCallback } from "react";
import { usePlayback } from "./usePlayback";

/**
 * Play one library track now.
 *
 * The daemon has no "play this track" mutation — the tracklist API can only
 * queue a track (`playNext`) or jump to a position it already holds. So this
 * queues the track directly after the current one and then skips onto it,
 * which is what the desktop's row click ends up doing through rockbox.
 */
export const usePlayTrack = () => {
  const { playNext, next, play, nowPlaying } = usePlayback();

  return useCallback(
    async (trackId: string) => {
      await playNext({ trackId });
      // With nothing playing there is no "current" to skip past, so the
      // queued track is already the one that starts.
      if (nowPlaying?.title) {
        await next();
      } else {
        await play();
      }
    },
    [playNext, next, play, nowPlaying?.title]
  );
};
