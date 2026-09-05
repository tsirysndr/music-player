import { FC } from "react";
import Sidebar from "./Sidebar";
import { useDevices } from "../../Hooks/useDevices";
import { usePlayback } from "../../Hooks/usePlayback";
import { usePlaylist } from "../../Hooks/usePlaylist";
import { useNavigate } from "react-router-dom";

export type SidebarProps = {
  active: string;
  playlists?: any[];
};

const SidebarWithData: FC<SidebarProps> = (props) => {
  const { devices, currentDevice, connectToDevice, disconnectFromDevice } =
    useDevices();

  const navigate = useNavigate();
  const { playPlaylist } = usePlayback();
  const {
    folders,
    mainPlaylists,
    createFolder,
    createPlaylist,
    deleteFolder,
    deletePlaylist,
    renameFolder,
    renamePlaylist,
  } = usePlaylist();

  return (
    <Sidebar
      {...props}
      onClickLibraryItem={(item) => navigate(`/${item}`)}
      onSearch={(query) => navigate(`/search?q=${query}`)}
      folders={folders}
      playlists={mainPlaylists}
      onCreateFolder={(name) => createFolder({ name })}
      onCreatePlaylist={(name) => createPlaylist({ name })}
      onDeleteFolder={(id) => deleteFolder({ id })}
      onDeletePlaylist={(id) => deletePlaylist({ id })}
      onEditFolder={(id, name) => renameFolder({ id, name })}
      onEditPlaylist={(id, name) => renamePlaylist({ id, name })}
      onPlayPlaylist={(playlistId, shuffle, position) =>
        playPlaylist({ playlistId, position, shuffle })
      }
      devices={devices}
      currentDevice={currentDevice}
      connectToDevice={(id) => connectToDevice({ id })}
      disconnectFromDevice={() => disconnectFromDevice()}
    />
  );
};

export default SidebarWithData;
