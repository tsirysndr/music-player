import { FC } from "react";
import Tracks from "./Tracks";
import { useGetTracksQuery } from "../../Hooks/GraphQL";
import { useDevices } from "../../Hooks/useDevices";
import { useTimeFormat } from "../../Hooks/useFormat";
import { usePlayback } from "../../Hooks/usePlayback";
import { usePlaylist } from "../../Hooks/usePlaylist";
import { resourceUriResolver } from "../../ResourceUriResolver";

const TracksWithData: FC = () => {
  const { data, loading } = useGetTracksQuery({
    fetchPolicy: "cache-and-network",
  });
  const { formatTime } = useTimeFormat();
  const { nowPlaying, playNext } = usePlayback();
  const { currentCastDevice } = useDevices();
  const tracks = !loading && data ? data.tracks : [];
  const { recentPlaylists, createPlaylist, addTrackToPlaylist } = usePlaylist();
  return (
    <Tracks
      tracks={tracks.slice(0, 100).map((track) => ({
        id: track.id,
        title: track.title,
        artist: track.artist,
        album: track.album.title,
        time: formatTime(track.duration! * 1000),
        cover: track.album.cover
          ? resourceUriResolver.resolve(`/covers/${track.album.cover}`)
          : undefined,
        artistId: track.artists[0].id,
        albumId: track.album.id,
      }))}
      nowPlaying={nowPlaying}
      onPlayTrack={(id, position) => {}}
      onPlayNext={(trackId) => playNext({ variables: { trackId } })}
      onCreatePlaylist={(name, description) =>
        createPlaylist({ variables: { name, description } })
      }
      onAddTrackToPlaylist={(playlistId, trackId) =>
        addTrackToPlaylist({ variables: { playlistId, trackId } })
      }
      recentPlaylists={recentPlaylists}
      currentCastDevice={currentCastDevice}
    />
  );
};

export default TracksWithData;
