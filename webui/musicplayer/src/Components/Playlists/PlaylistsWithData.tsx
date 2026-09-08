import { FC } from "react";
import { usePlayback } from "../../Hooks/usePlayback";
import { usePlaylist } from "../../Hooks/usePlaylist";
import Playlists from "./Playlists";

const PlaylistsWithData: FC = () => {
  const {
    playlists,
    folders,
    createPlaylist,
    createFolder,
    renamePlaylist,
    deletePlaylist,
  } = usePlaylist();
  const { playPlaylist } = usePlayback();

  return (
    <Playlists
      playlists={playlists.map((playlist) => ({
        id: playlist.id,
        name: playlist.name,
        description: playlist.description,
        trackCount: playlist.trackCount,
      }))}
      folders={folders}
      onCreatePlaylist={(name, description, smart) =>
        createPlaylist({ name, description, smart })
      }
      onCreateFolder={(name) => createFolder({ name })}
      onEditPlaylist={(id, name) => renamePlaylist({ id, name })}
      onDeletePlaylist={(id) => deletePlaylist({ id })}
      onPlayPlaylist={(playlistId) =>
        playPlaylist({ playlistId, shuffle: false })
      }
    />
  );
};

export default PlaylistsWithData;
