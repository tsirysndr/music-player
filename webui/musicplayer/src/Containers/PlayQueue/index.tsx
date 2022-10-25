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
import { useTimeFormat } from "../../Hooks/useFormat";
import { usePlayback } from "../../Hooks/usePlayback";

const PlayQueuePage = () => {
  const navigate = useNavigate();
  const { data, loading } = useGetTracklistQuery();
  const { play, pause, next, previous, nowPlaying } = usePlayback();
  const { formatTime } = useTimeFormat();
  const previousTracks = (
    !loading && data ? data.tracklistTracks.previousTracks : []
  ).map((track) => ({
    id: track.id,
    title: track.title,
    duration: track.duration,
    album: track.album.title,
    artist: track.artists.map((artist) => artist.name).join(", "),
    time: formatTime(track.duration! * 1000),
  }));
  const nextTracks = (
    !loading && data ? data.tracklistTracks.nextTracks : []
  ).map((track) => ({
    id: track.id,
    title: track.title,
    duration: track.duration,
    album: track.album.title,
    artist: track.artists.map((artist) => artist.name).join(", "),
    time: formatTime(track.duration! * 1000),
  }));
  return (
    <PlayQueue
      tracks={[...previousTracks, ...nextTracks]}
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
