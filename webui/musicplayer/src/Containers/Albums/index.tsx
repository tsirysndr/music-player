import { useNavigate } from "react-router-dom";
import Albums from "../../Components/Albums";
import { useGetAlbumsQuery } from "../../Hooks/GraphQL";
import { usePlayback } from "../../Hooks/usePlayback";
import { usePlaylist } from "../../Hooks/usePlaylist";
import { useSearch } from "../../Hooks/useSearch";

const AlbumsPage = () => {
  const { data, loading } = useGetAlbumsQuery({
    variables: {
      limit: 10000,
    },
    fetchPolicy: "cache-and-network",
  });
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
  const navigate = useNavigate();
  const { onSearch } = useSearch();
  const albums = !loading && data ? data.albums : [];
  const {
    folders,
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
    <Albums
      albums={albums.map((album) => ({
        id: album.id,
        title: album.title,
        artist: album.artist,
        cover: album.cover && `/covers/${album.cover}`,
      }))}
      onClickAlbum={({ id }) => navigate(`/albums/${id}`)}
      onClickLibraryItem={(item) => navigate(`/${item}`)}
      onPlay={() => play()}
      onPause={() => pause()}
      onNext={() => next()}
      onPrevious={() => previous()}
      onShuffle={() => {}}
      onRepeat={() => {}}
      nextTracks={nextTracks}
      previousTracks={previousTracks}
      onPlayNext={(trackId) => playNext({ variables: { trackId } })}
      onPlayTrackAt={(position) => playTrackAt({ variables: { position } })}
      onRemoveTrackAt={(position) => removeTrackAt({ variables: { position } })}
      onSearch={(query) => navigate(`/search?q=${query}`)}
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
      onPlayPlaylist={(playlistId, shuffle, position) =>
        playPlaylist({ variables: { playlistId, position, shuffle } })
      }
    />
  );
};

export default AlbumsPage;
