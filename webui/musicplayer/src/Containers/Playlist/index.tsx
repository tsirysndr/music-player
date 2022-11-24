import { useNavigate, useParams } from "react-router-dom";
import Playlist from "../../Components/Playlist";
import { useGetPlaylistQuery } from "../../Hooks/GraphQL";
import { useTimeFormat } from "../../Hooks/useFormat";
import { usePlaylist } from "../../Hooks/usePlaylist";

const PlaylistPage = () => {
  const params = useParams();
  const { data, loading, refetch } = useGetPlaylistQuery({
    variables: {
      id: params.id!,
      limit: 100000,
    },
    fetchPolicy: "network-only",
  });
  const navigate = useNavigate();
  const { formatTime } = useTimeFormat();

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
    <Playlist
      onBack={() => navigate(-1)}
      onClickLibraryItem={(item) => navigate(`/${item}`)}
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
      onAddTrackToPlaylist={(trackId, playlistId) =>
        addTrackToPlaylist({ variables: { trackId, playlistId } })
      }
      playlist={data?.playlist}
      recentPlaylists={recentPlaylists}
    />
  );
};

export default PlaylistPage;
