import { FC, useEffect } from "react";
import AlbumDetails from "./AlbumDetails";
import { useGetAlbumQuery } from "../../Hooks/GraphQL";
import { useTimeFormat } from "../../Hooks/useFormat";
import { usePlayback } from "../../Hooks/usePlayback";
import { usePlaylist } from "../../Hooks/usePlaylist";
import { useNavigate, useParams } from "react-router-dom";

const AlbumDetailsWithData: FC = () => {
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
  const { nowPlaying, playAlbum, playNext } = usePlayback();
  const album =
    !loading && data
      ? {
          ...data.album,
          tracks: data.album.tracks.map((track, index) => ({
            "#": track.trackNumber,
            id: track.id,
            title: track.title,
            artist: track.artists.map((artist) => artist.name).join(", "),
            time: formatTime(track.duration! * 1000),
            artistId: track.artists[0].id,
            albumId: data.album.id,
            index,
          })),
        }
      : { tracks: [] };
  const { recentPlaylists, createPlaylist, addTrackToPlaylist } = usePlaylist();
  return (
    <AlbumDetails
      onBack={() => navigate(-1)}
      album={album}
      nowPlaying={nowPlaying}
      onPlayAlbum={(albumId, shuffle, position) =>
        playAlbum({ variables: { albumId, position, shuffle } })
      }
      onPlayNext={(trackId) => playNext({ variables: { trackId } })}
      onCreatePlaylist={(name) => createPlaylist({ variables: { name } })}
      onAddTrackToPlaylist={(playlistId, trackId) =>
        addTrackToPlaylist({ variables: { playlistId, trackId } })
      }
      recentPlaylists={recentPlaylists}
    />
  );
};

export default AlbumDetailsWithData;
