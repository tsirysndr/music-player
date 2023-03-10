import { useEffect, useMemo, FC } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import SearchResults from "./SearchResults";
import { useDevices } from "../../Hooks/useDevices";
import { useTimeFormat } from "../../Hooks/useFormat";
import { usePlayback } from "../../Hooks/usePlayback";
import { usePlaylist } from "../../Hooks/usePlaylist";
import { useSearch } from "../../Hooks/useSearch";

const SearchResultsWithData: FC = () => {
  const navigate = useNavigate();
  const { formatTime } = useTimeFormat();
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
  const [params] = useSearchParams();
  const { onSearch, results, query } = useSearch();
  const q = useMemo(() => params.get("q"), [params]);

  useEffect(() => {
    if (q && q !== null) {
      onSearch(q);
    }
  }, [q]);

  return (
    <SearchResults
      tracks={results.tracks.map((x) => ({
        ...x,
        time: formatTime(x.duration * 1000),
      }))}
      albums={results.albums}
      artists={results.artists}
      onClickAlbum={({ id }) => navigate(`/albums/${id}`)}
      onClickArtist={({ id }) => navigate(`/artists/${id}`)}
      onClickLibraryItem={(item) => navigate(`/${item}`)}
      nowPlaying={nowPlaying}
      onPlayTrack={(id, position) => {}}
      nextTracks={nextTracks}
      previousTracks={previousTracks}
      onPlayNext={(trackId) => playNext({ variables: { trackId } })}
      onPlayTrackAt={(position) => playTrackAt({ variables: { position } })}
      onRemoveTrackAt={(position) => removeTrackAt({ variables: { position } })}
      onSearch={onSearch}
      folders={folders}
      playlists={mainPlaylists}
      onCreateFolder={(name) => createFolder({ variables: { name } })}
      onCreatePlaylist={(name, description) =>
        createPlaylist({ variables: { name, description } })
      }
      onDeleteFolder={(id) => deleteFolder({ variables: { id } })}
      onDeletePlaylist={(id) => deletePlaylist({ variables: { id } })}
      onEditFolder={(id, name) => renameFolder({ variables: { id, name } })}
      onEditPlaylist={(id, name, description) =>
        renamePlaylist({ variables: { id, name } })
      }
      onAddTrackToPlaylist={(playlistId, trackId) =>
        addTrackToPlaylist({ variables: { trackId, playlistId } })
      }
      onPlayPlaylist={(playlistId, shuffle, position) =>
        playPlaylist({ variables: { playlistId, position, shuffle } })
      }
      recentPlaylists={recentPlaylists}
      currentDevice={currentDevice}
      currentCastDevice={currentCastDevice}
    />
  );
};

export default SearchResultsWithData;
