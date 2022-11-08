import { useNavigate } from "react-router-dom";
import Tracks from "../../Components/Tracks";
import { useGetTracksQuery } from "../../Hooks/GraphQL";
import { useTimeFormat } from "../../Hooks/useFormat";
import { usePlayback } from "../../Hooks/usePlayback";

const TracksPage = () => {
  const { data, loading } = useGetTracksQuery({
    fetchPolicy: "cache-and-network",
  });
  const navigate = useNavigate();
  const { formatTime } = useTimeFormat();
  const {
    play,
    pause,
    next,
    previous,
    nowPlaying,
    nextTracks,
    previousTracks,
    playNext,
    playTrackAt,
    removeTrackAt,
  } = usePlayback();
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
          cover: track.album.cover ? `/covers/${track.album.cover}` : undefined,
          artistId: track.artists[0].id,
          albumId: track.album.id,
        }))}
        onClickLibraryItem={(item) => navigate(`/${item}`)}
        onPlay={() => play()}
        onPause={() => pause()}
        onNext={() => next()}
        onPrevious={() => previous()}
        onShuffle={() => {}}
        onRepeat={() => {}}
        nowPlaying={nowPlaying}
        onPlayTrack={(id, position) => {}}
        nextTracks={nextTracks}
        previousTracks={previousTracks}
        onPlayNext={(trackId) => playNext({ variables: { trackId } })}
        onPlayTrackAt={(position) => playTrackAt({ variables: { position } })}
        onRemoveTrackAt={(position) =>
          removeTrackAt({ variables: { position } })
        }
      />
    </>
  );
};

export default TracksPage;
