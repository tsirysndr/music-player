import { useEffect } from "react";
import {
  useCurrentlyPlayingSongQuery,
  usePlayMutation,
  usePauseMutation,
  useNextMutation,
  usePreviousMutation,
} from "./GraphQL";

export const usePlayback = () => {
  const {
    data: playback,
    startPolling,
    stopPolling,
  } = useCurrentlyPlayingSongQuery({
    pollInterval: 500,
  });
  const [play] = usePlayMutation();
  const [pause] = usePauseMutation();
  const [next] = useNextMutation();
  const [previous] = usePreviousMutation();
  const index = playback?.currentlyPlayingSong.index;
  const duration = playback?.currentlyPlayingSong?.track?.duration! * 1000;
  const position = playback?.currentlyPlayingSong?.positionMs!;
  const nowPlaying = {
    id: playback?.currentlyPlayingSong?.track?.id,
    title: playback?.currentlyPlayingSong?.track?.title,
    artist: playback?.currentlyPlayingSong?.track?.artists
      ?.map((artist) => artist.name)
      .join(", "),
    album: playback?.currentlyPlayingSong?.track?.album?.title,
    isPlaying: playback?.currentlyPlayingSong?.isPlaying,
    duration,
    progress: position,
    cover: `/covers/${playback?.currentlyPlayingSong?.track?.album?.id}.jpg`,
    albumId: playback?.currentlyPlayingSong?.track?.album?.id,
  };
  useEffect(() => {
    startPolling!(1000);
    return () => stopPolling();
  }, [startPolling, stopPolling]);
  return {
    nowPlaying,
    play,
    pause,
    next,
    previous,
    index,
  };
};
