import { useNavigate } from "react-router-dom";
import Folder from "../../Components/Folder";
import { useTimeFormat } from "../../Hooks/useFormat";
import { usePlayback } from "../../Hooks/usePlayback";
import { usePlaylist } from "../../Hooks/usePlaylist";
import { useSearch } from "../../Hooks/useSearch";

const FolderPage = () => {
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
    folders,
    playlists,
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
    <Folder
      onBack={() => navigate(-1)}
      onClickLibraryItem={(item) => navigate(`/${item}`)}
      onPlay={() => play()}
      onPause={() => pause()}
      onNext={() => next()}
      onPrevious={() => previous()}
      onShuffle={() => {}}
      onRepeat={() => {}}
      nowPlaying={nowPlaying}
      nextTracks={nextTracks}
      previousTracks={previousTracks}
      onPlayNext={(trackId) => playNext({ variables: { trackId } })}
      onPlayTrackAt={(position) => playTrackAt({ variables: { position } })}
      onRemoveTrackAt={(position) => removeTrackAt({ variables: { position } })}
      onSearch={(query) => navigate(`/search?q=${query}`)}
      folders={folders}
      playlists={playlists}
      onCreateFolder={(name) => createFolder({ variables: { name } })}
      onCreatePlaylist={(name, description) =>
        createPlaylist({ variables: { name, description } })
      }
      onDeleteFolder={(id) => deleteFolder({ variables: { id } })}
      onDeletePlaylist={(id) => deletePlaylist({ variables: { id } })}
      onEditFolder={(id, name) => renameFolder({ variables: { id, name } })}
      onEditPlaylist={(id, name) => renamePlaylist({ variables: { id, name } })}
      onPlayPlaylist={(playlistId, shuffle, position) =>
        playPlaylist({ variables: { playlistId, position, shuffle } })
      }
    />
  );
};

export default FolderPage;
