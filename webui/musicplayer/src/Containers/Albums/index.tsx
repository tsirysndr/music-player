import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import Albums from "../../Components/Albums";
import {
  useCurrentlyPlayingSongQuery,
  useGetAlbumsQuery,
  useNextMutation,
  usePauseMutation,
  usePlayMutation,
  usePreviousMutation,
} from "../../Hooks/GraphQL";

const AlbumsPage = () => {
  const { data: albums } = useGetAlbumsQuery();
  const {
    data: playback,
    startPolling,
    stopPolling,
  } = useCurrentlyPlayingSongQuery();
  const [play] = usePlayMutation();
  const [pause] = usePauseMutation();
  const [next] = useNextMutation();
  const [previous] = usePreviousMutation();
  const navigate = useNavigate();
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
    <Albums
      albums={[]}
      onClickAlbum={() => {}}
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

export default AlbumsPage;
