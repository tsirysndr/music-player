import { FC } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { useGetPlaylistQuery } from "../../Hooks/GraphQL";
import { useTimeFormat } from "../../Hooks/useFormat";
import { useLikes } from "../../Hooks/useLikes";
import { usePlayback } from "../../Hooks/usePlayback";
import { usePlaylist } from "../../Hooks/usePlaylist";
import Playlist from "./Playlist";

const PlaylistWithData: FC = () => {
  const params = useParams();
  const { data, isLoading: loading } = useGetPlaylistQuery({ id: params.id! });
  const navigate = useNavigate();
  const { formatTime } = useTimeFormat();
  const { nowPlaying, playNext, playPlaylist } = usePlayback();
  const { isLiked, toggleLike } = useLikes();
  const { recentPlaylists, addTrackToPlaylist, removeTrackFromPlaylist } =
    usePlaylist();

  const playlist = data?.playlist && {
    id: data.playlist.id,
    name: data.playlist.name,
    description: data.playlist.description,
    tracks: data.playlist.tracks.map((track) => ({
      id: track.id,
      title: track.title,
      artist: track.artist,
      artistId: track.artistId,
      album: track.albumTitle,
      albumId: track.albumId,
      duration: formatTime((track.duration ?? 0) * 1000),
      key: track.key,
      bpm: track.bpm,
      liked: isLiked(track.id),
    })),
  };

  return (
    <Playlist
      playlist={playlist}
      loading={loading}
      currentTrackId={nowPlaying?.isPlaying ? nowPlaying.id : undefined}
      recentPlaylists={recentPlaylists}
      onBack={() => navigate(-1)}
      onPlayPlaylist={(playlistId, shuffle, position) =>
        playPlaylist({ playlistId, position, shuffle })
      }
      onPlayNext={(trackId) => playNext({ trackId })}
      onToggleLike={toggleLike}
      onRemoveTrack={(position) =>
        removeTrackFromPlaylist({ playlistId: params.id!, position })
      }
      onAddTrackToPlaylist={(playlistId, trackId) =>
        addTrackToPlaylist({ playlistId, trackId })
      }
    />
  );
};

export default PlaylistWithData;
