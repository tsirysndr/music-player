import { FC, useEffect } from "react";
import { useParams } from "react-router-dom";
import Folder from "./Folder";
import { useGetFolderQuery } from "../../Hooks/GraphQL";
import { useDevices } from "../../Hooks/useDevices";
import { usePlayback } from "../../Hooks/usePlayback";
import { usePlaylist } from "../../Hooks/usePlaylist";

const FolderWithData: FC = () => {
  const params = useParams();
  const { data, refetch } = useGetFolderQuery({
    variables: {
      id: params.id!,
    },
    fetchPolicy: "network-only",
  });

  useEffect(() => {
    params.id && refetch();
  }, [params.id, refetch]);
  const { playNext } = usePlayback();
  const { currentCastDevice } = useDevices();
  const { playlists, mainPlaylists, createPlaylist, movePlaylistsToFolder } =
    usePlaylist();
  return (
    <Folder
      onPlayNext={(trackId) => playNext({ variables: { trackId } })}
      playlists={playlists}
      mainPlaylists={mainPlaylists}
      onCreatePlaylist={(name, description) =>
        createPlaylist({ variables: { name, description } })
      }
      onMovePlaylists={(playlistIds, folderId) =>
        movePlaylistsToFolder({ variables: { playlistIds, folderId } })
      }
      folder={data?.folder}
      currentCastDevice={currentCastDevice}
    />
  );
};

export default FolderWithData;
