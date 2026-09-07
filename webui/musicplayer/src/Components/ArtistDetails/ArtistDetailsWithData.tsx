import { FC } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { useGetArtistQuery } from "../../Hooks/GraphQL";
import { useTimeFormat } from "../../Hooks/useFormat";
import { useLikes } from "../../Hooks/useLikes";
import { usePlayback } from "../../Hooks/usePlayback";
import { usePlaylist } from "../../Hooks/usePlaylist";
import ArtistDetails from "./ArtistDetails";

const ArtistDetailsWithData: FC = () => {
  const params = useParams();
  const { data, isLoading: loading } = useGetArtistQuery({ id: params.id! });
  const navigate = useNavigate();
  const { formatTime } = useTimeFormat();
  const { nowPlaying, playArtistTracks, playAlbum, playNext } = usePlayback();
  const { isLiked, toggleLike } = useLikes();
  const { recentPlaylists, addTrackToPlaylist } = usePlaylist();

  const artist = data?.artist && {
    id: data.artist.id,
    name: data.artist.name,
    picture: data.artist.picture,
  };

  const albums = (data?.artist.albums ?? []).map((album) => ({
    id: album.id,
    title: album.title,
    artist: album.artist,
    year: album.year,
    cover: album.cover ? `/covers/${album.cover}` : undefined,
  }));

  const tracks = (data?.artist.songs ?? []).map((track) => ({
    id: track.id,
    title: track.title,
    artist: track.artist,
    artistId: track.artists[0]?.id,
    album: track.album.title,
    albumId: track.album.id,
    duration: formatTime((track.duration ?? 0) * 1000),
    liked: isLiked(track.id),
  }));

  return (
    <ArtistDetails
      artist={artist}
      albums={albums}
      tracks={tracks}
      loading={loading}
      currentTrackId={nowPlaying?.isPlaying ? nowPlaying.id : undefined}
      recentPlaylists={recentPlaylists}
      onBack={() => navigate(-1)}
      onPlayArtist={(artistId, shuffle, position) =>
        playArtistTracks({ artistId, position, shuffle })
      }
      onPlayAlbum={(albumId, shuffle) => playAlbum({ albumId, shuffle })}
      onPlayNext={(trackId) => playNext({ trackId })}
      onToggleLike={toggleLike}
      onAddTrackToPlaylist={(playlistId, trackId) =>
        addTrackToPlaylist({ playlistId, trackId })
      }
    />
  );
};

export default ArtistDetailsWithData;
