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
  const { currentDevice, currentCastDevice } = useDevices();
  const {
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
    recentPlaylists,
    mainPlaylists,
    createFolder,
    createPlaylist,
    addTrackToPlaylist,
    deleteFolder,
    deletePlaylist,
    renameFolder,
    renamePlaylist,
  } = usePlaylist();
  const [params] = useSearchParams();
  const { onSearch, results } = useSearch();
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
      onPlayNext={(trackId) => playNext({ trackId })}
      onPlayTrackAt={(position) => playTrackAt({ position })}
      onRemoveTrackAt={(position) => removeTrackAt({ position })}
      onSearch={onSearch}
      folders={folders}
      playlists={mainPlaylists}
      onCreateFolder={(name) => createFolder({ name })}
      onCreatePlaylist={(name, description) =>
        createPlaylist({ name, description })
      }
      onDeleteFolder={(id) => deleteFolder({ id })}
      onDeletePlaylist={(id) => deletePlaylist({ id })}
      onEditFolder={(id, name) => renameFolder({ id, name })}
      onEditPlaylist={(id, name, description) =>
        renamePlaylist({ id, name })
      }
      onAddTrackToPlaylist={(playlistId, trackId) =>
        addTrackToPlaylist({ trackId, playlistId })
      }
      onPlayPlaylist={(playlistId, shuffle, position) =>
        playPlaylist({ playlistId, position, shuffle })
      }
      recentPlaylists={recentPlaylists}
      currentDevice={currentDevice}
      currentCastDevice={currentCastDevice}
    />
  );
};

export default SearchResultsWithData;
