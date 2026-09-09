import { FC } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { useAddTracksMutation, useGetAlbumQuery } from "../../Hooks/GraphQL";
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
  const addTracks = useAddTracksMutation();

  /**
   * Queue the whole album, the way the desktop's album menu does.
   *
   * `-2` is "play next" and `-3` is "add to queue" — the engine's own
   * positions. There is no id-list mutation, so play-next walks the album
   * backwards (each insert lands directly after the current track, so the last
   * one inserted ends up first) and add-to-queue sends the tracks in one go.
   */
  const queueAlbum = async (position: -2 | -3) => {
    const tracks = data?.album?.tracks ?? [];
    if (position === -2) {
      for (const track of [...tracks].reverse()) {
        await playNext({ trackId: track.id });
      }
      return;
    }
    await addTracks.mutateAsync({
      tracks: tracks.map((track) => ({
        id: track.id,
        title: track.title,
        uri: track.uri,
        duration: track.duration,
        discNumber: track.discNumber,
        trackNumber: track.trackNumber,
      })),
    });
  };

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
      discNumber: track.discNumber,
      duration: formatTime((track.duration ?? 0) * 1000),
      key: track.key,
      bpm: track.bpm,
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
      onQueueAlbum={(_albumId, position) => queueAlbum(position)}
      // Liking an album is liking every track on it — the desktop does the
      // same, because a like lives on the track.
      onLikeAlbum={() =>
        (data?.album?.tracks ?? [])
          .filter((track) => !isLiked(track.id))
          .forEach((track) => toggleLike(track.id))
      }
    />
  );
};

export default AlbumDetailsWithData;
