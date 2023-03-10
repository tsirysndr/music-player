import { FC } from "react";
import Tracks from "./Tracks";
import { useNavigate } from "react-router-dom";
import { useGetTracksQuery } from "../../Hooks/GraphQL";
import { useDevices } from "../../Hooks/useDevices";
import { useTimeFormat } from "../../Hooks/useFormat";
import { usePlayback } from "../../Hooks/usePlayback";
import { usePlaylist } from "../../Hooks/usePlaylist";
import { useSearch } from "../../Hooks/useSearch";
import { resourceUriResolver } from "../../ResourceUriResolver";

const TracksWithData: FC = () => {
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
    playPlaylist,
  } = usePlayback();
  const { onSearch } = useSearch();
  const {
    devices,
    castDevices,
    currentDevice,
    currentCastDevice,
    connectToDevice,
    disconnectFromDevice,
    connectToCastDevice,
    disconnectFromCastDevice,
  } = useDevices();
  const tracks = !loading && data ? data.tracks : [];
  const {
    folders,
    playlists,
    recentPlaylists,
    mainPlaylists,
    createFolder,
    createPlaylist,
    addTrackToPlaylist,
    movePlaylistToFolder,
    deleteFolder,
    deletePlaylist,
    renameFolder,
    renamePlaylist,
  } = usePlaylist();
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
      onRemoveTrackAt={(position) => removeTrackAt({ variables: { position } })}
      onSearch={(query) => navigate(`/search?q=${query}`)}
      playlists={mainPlaylists}
      folders={folders}
      onCreateFolder={(name) => createFolder({ variables: { name } })}
      onCreatePlaylist={(name, description) =>
        createPlaylist({ variables: { name, description } })
      }
      onAddTrackToPlaylist={(playlistId, trackId) =>
        addTrackToPlaylist({ variables: { playlistId, trackId } })
      }
      onDeleteFolder={(id) => deleteFolder({ variables: { id } })}
      onDeletePlaylist={(id) => deletePlaylist({ variables: { id } })}
      onEditFolder={(id, name) => renameFolder({ variables: { id, name } })}
      onEditPlaylist={(id, name, description) =>
        renamePlaylist({ variables: { id, name } })
      }
      onPlayPlaylist={(playlistId, shuffle, position) =>
        playPlaylist({ variables: { playlistId, position, shuffle } })
      }
      recentPlaylists={recentPlaylists}
      devices={devices}
      castDevices={castDevices}
      currentDevice={currentDevice}
      currentCastDevice={currentCastDevice}
      connectToDevice={(id) => connectToDevice({ variables: { id } })}
      disconnectFromDevice={() => disconnectFromDevice()}
      connectToCastDevice={(id) => connectToCastDevice({ variables: { id } })}
      disconnectFromCastDevice={() => disconnectFromCastDevice()}
    />
  );
};

export default TracksWithData;
