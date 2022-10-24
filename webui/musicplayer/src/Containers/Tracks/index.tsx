import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import Tracks from "../../Components/Tracks";
import {
  useCurrentlyPlayingSongQuery,
  useGetTracksQuery,
  useNextMutation,
  usePauseMutation,
  usePlayMutation,
  usePreviousMutation,
} from "../../Hooks/GraphQL";
import { useTimeFormat } from "../../Hooks/useFormat";

const TracksPage = () => {
  const { data, loading } = useGetTracksQuery();
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
  const navigate = useNavigate();
  const { formatTime } = useTimeFormat();
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
  const tracks = !loading && data ? data.tracks : [];
  return (
    <>
      <Tracks
        tracks={tracks.slice(0, 100).map((track) => ({
          id: track.id,
          title: track.title,
          artist: track.artists.map((artist) => artist.name).join(", "),
          album: track.album.title,
          time: formatTime(track.duration! * 1000),
        }))}
        onClickLibraryItem={(item) => navigate(`/${item}`)}
        onPlay={() => play()}
        onPause={() => pause()}
        onNext={() => next()}
        onPrevious={() => previous()}
        onShuffle={() => {}}
        onRepeat={() => {}}
        nowPlaying={nowPlaying}
      />
    </>
  );
};

export default TracksPage;
