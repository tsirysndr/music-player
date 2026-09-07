import { FC } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { useGetAlbumQuery } from "../../Hooks/GraphQL";
import { useTimeFormat } from "../../Hooks/useFormat";
import { useLikes } from "../../Hooks/useLikes";
import { usePlayback } from "../../Hooks/usePlayback";
import { usePlaylist } from "../../Hooks/usePlaylist";
import AlbumDetails from "./AlbumDetails";

const AlbumDetailsWithData: FC = () => {
  const params = useParams();
  const { data, isLoading: loading } = useGetAlbumQuery({ id: params.id! });
  const navigate = useNavigate();
  const { formatTime } = useTimeFormat();
  const { nowPlaying, playAlbum, playNext } = usePlayback();
  const { isLiked, toggleLike } = useLikes();
  const { recentPlaylists, addTrackToPlaylist } = usePlaylist();

  const album = data?.album && {
    id: data.album.id,
    title: data.album.title,
    artist: data.album.artist,
    year: data.album.year,
    cover: data.album.cover ? `/covers/${data.album.cover}` : undefined,
    meta:
      data.album.tracks.length === 1
        ? "1 track"
        : `${data.album.tracks.length} tracks`,
    tracks: data.album.tracks.map((track) => ({
      id: track.id,
      title: track.title,
      artist: track.artists.map((artist) => artist.name).join(", "),
      artistId: track.artists[0]?.id,
      album: data.album.title,
      albumId: data.album.id,
      trackNumber: track.trackNumber,
      duration: formatTime((track.duration ?? 0) * 1000),
      liked: isLiked(track.id),
    })),
  };

  return (
    <AlbumDetails
      album={album}
      loading={loading}
      currentTrackId={nowPlaying?.isPlaying ? nowPlaying.id : undefined}
      recentPlaylists={recentPlaylists}
      onBack={() => navigate(-1)}
      onPlayAlbum={(albumId, shuffle, position) =>
        playAlbum({ albumId, position, shuffle })
      }
      onPlayNext={(trackId) => playNext({ trackId })}
      onToggleLike={toggleLike}
      onAddTrackToPlaylist={(playlistId, trackId) =>
        addTrackToPlaylist({ playlistId, trackId })
      }
    />
  );
};

export default AlbumDetailsWithData;
