import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import PlayQueue from "../../Components/PlayQueue";
import {
  useCurrentlyPlayingSongQuery,
  useGetTracklistQuery,
  useNextMutation,
  usePauseMutation,
  usePlayMutation,
  usePreviousMutation,
} from "../../Hooks/GraphQL";

const PlayQueuePage = () => {
  const navigate = useNavigate();
  const { data, loading } = useGetTracklistQuery();
  const {
    data: playback,
    startPolling,
    stopPolling,
  } = useCurrentlyPlayingSongQuery({
    pollInterval: 1000,
    nextFetchPolicy: "network-only",
  });
  const [play] = usePlayMutation();
  const [pause] = usePauseMutation();
  const [next] = useNextMutation();
  const [previous] = usePreviousMutation();
  const duration = playback?.currentlyPlayingSong?.track?.duration! * 1000;
  const position = playback?.currentlyPlayingSong?.positionMs!;
  const nowPlaying = {
    title: playback?.currentlyPlayingSong?.track?.title,
    artist: playback?.currentlyPlayingSong?.track?.artists
      ?.map((artist) => artist.name)
      .join(", "),
    album: playback?.currentlyPlayingSong?.track?.album?.title,
    isPlaying: playback?.currentlyPlayingSong?.isPlaying,
    duration,
    progress: position,
  };
  useEffect(() => {
    startPolling!(1000);
    return () => stopPolling();
  }, [startPolling, stopPolling]);
  return (
    <PlayQueue
      tracks={[]}
      onClickLibraryItem={(item) => navigate(`/${item}`)}
      onPlay={() => play()}
      onPause={() => pause()}
      onNext={() => next()}
      onPrevious={() => previous()}
      onShuffle={() => {}}
      onRepeat={() => {}}
      nowPlaying={nowPlaying}
    />
  );
};

export default PlayQueuePage;
