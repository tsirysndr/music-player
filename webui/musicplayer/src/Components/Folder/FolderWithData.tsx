import { FC } from "react";
import { useParams } from "react-router-dom";
import Folder from "./Folder";
import { useGetFolderQuery } from "../../Hooks/GraphQL";
import { useDevices } from "../../Hooks/useDevices";
import { usePlayback } from "../../Hooks/usePlayback";
import { usePlaylist } from "../../Hooks/usePlaylist";

const FolderWithData: FC = () => {
  const params = useParams();
  const { data } = useGetFolderQuery({
    id: params.id!,
  });

  const { playNext } = usePlayback();
  const { currentCastDevice } = useDevices();
  const { playlists, mainPlaylists, createPlaylist, movePlaylistsToFolder } =
    usePlaylist();

  const onFilter = (value: string) => {};

  return (
    <Folder
      onPlayNext={(trackId) => playNext({ trackId })}
      playlists={playlists}
      mainPlaylists={mainPlaylists}
      onCreatePlaylist={(name, description) =>
        createPlaylist({ name, description })
      }
      onMovePlaylists={(playlistIds, folderId) =>
        movePlaylistsToFolder({ playlistIds, folderId })
      }
      folder={data?.folder}
      currentCastDevice={currentCastDevice}
      onFilter={onFilter}
    />
  );
};

export default FolderWithData;
