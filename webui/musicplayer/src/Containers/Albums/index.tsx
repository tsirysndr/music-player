import { useNavigate } from "react-router-dom";
import Albums from "../../Components/Albums";
import { useGetAlbumsQuery } from "../../Hooks/GraphQL";
import { usePlayback } from "../../Hooks/usePlayback";
import { usePlaylist } from "../../Hooks/usePlaylist";
import { useSearch } from "../../Hooks/useSearch";

const AlbumsPage = () => {
  const { data, loading } = useGetAlbumsQuery({
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
  } = usePlayback();
  const navigate = useNavigate();
  const { onSearch } = useSearch();
  const albums = !loading && data ? data.albums : [];
  const {
    folders,
    playlists,
    createFolder,
    createPlaylist,
    addTrackToPlaylist,
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

export default AlbumsPage;
