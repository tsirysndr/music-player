import { useNavigate } from "react-router-dom";
import Playlist from "../../Components/Playlist";
import { useTimeFormat } from "../../Hooks/useFormat";
import { usePlayback } from "../../Hooks/usePlayback";
import { usePlaylist } from "../../Hooks/usePlaylist";
import { useSearch } from "../../Hooks/useSearch";

const PlaylistPage = () => {
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
  } = usePlayback();
  const { onSearch } = useSearch();
  const { folders, playlists, createFolder, createPlaylist } = usePlaylist();
  return (
    <Playlist
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
    />
  );
};

export default PlaylistPage;
