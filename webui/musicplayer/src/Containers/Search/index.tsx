import { useEffect, useMemo } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import SearchResults from "../../Components/SearchResults";
import { useDevices } from "../../Hooks/useDevices";
import { useTimeFormat } from "../../Hooks/useFormat";
import { usePlayback } from "../../Hooks/usePlayback";
import { usePlaylist } from "../../Hooks/usePlaylist";
import { useSearch } from "../../Hooks/useSearch";

const SearchPage = () => {
  const navigate = useNavigate();
  const { formatTime } = useTimeFormat();
  const {
    devices,
    castDevices,
    currentDevice,
    connectToDevice,
    disconnectFromDevice,
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
    <>
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
        onRemoveTrackAt={(position) =>
          removeTrackAt({ variables: { position } })
        }
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
        onAddTrackToPlaylist={(trackId, playlistId) =>
          addTrackToPlaylist({ variables: { trackId, playlistId } })
        }
        onPlayPlaylist={(playlistId, shuffle, position) =>
          playPlaylist({ variables: { playlistId, position, shuffle } })
        }
        recentPlaylists={recentPlaylists}
        devices={devices}
        castDevices={castDevices}
        currentDevice={currentDevice}
        connectToDevice={(id) => connectToDevice({ variables: { id } })}
        disconnectFromDevice={() => disconnectFromDevice()}
      />
    </>
  );
};

export default SearchPage;
