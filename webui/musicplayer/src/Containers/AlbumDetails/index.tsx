import { useEffect } from "react";
import { useNavigate, useParams } from "react-router-dom";
import AlbumDetails from "../../Components/AlbumDetails";
import { useGetAlbumQuery } from "../../Hooks/GraphQL";
import { useTimeFormat } from "../../Hooks/useFormat";
import { usePlayback } from "../../Hooks/usePlayback";

const AlbumDetailsPage = () => {
  const params = useParams();
  const { data, loading, refetch } = useGetAlbumQuery({
    variables: {
      id: params.id!,
    },
    fetchPolicy: "network-only",
  });

  useEffect(() => {
    params.id && refetch();
  }, [params.id, refetch]);

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
    playAlbum,
    playNext,
    playTrackAt,
    removeTrackAt,
  } = usePlayback();
  const album =
    !loading && data
      ? {
          ...data.album,
          tracks: data.album.tracks.map((track) => ({
            "#": track.trackNumber,
            id: track.id,
            title: track.title,
            artist: track.artists.map((artist) => artist.name).join(", "),
            time: formatTime(track.duration! * 1000),
            artistId: track.artists[0].id,
          })),
        }
      : { tracks: [] };
  return (
    <AlbumDetails
      onBack={() => navigate(-1)}
      onClickLibraryItem={(item) => navigate(`/${item}`)}
      onClickAlbum={() => {}}
      onPlay={() => play()}
      onPause={() => pause()}
      onNext={() => next()}
      onPrevious={() => previous()}
      onShuffle={() => {}}
      onRepeat={() => {}}
      album={album}
      nowPlaying={nowPlaying}
      nextTracks={nextTracks}
      previousTracks={previousTracks}
      onPlayAlbum={(albumId, shuffle, position) =>
        playAlbum({ variables: { albumId, position, shuffle } })
      }
      onPlayNext={(trackId) => playNext({ variables: { trackId } })}
      onPlayTrackAt={(position) => playTrackAt({ variables: { position } })}
      onRemoveTrackAt={(position) => removeTrackAt({ variables: { position } })}
    />
  );
};

export default AlbumDetailsPage;
