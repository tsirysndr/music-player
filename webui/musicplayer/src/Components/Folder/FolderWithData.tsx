import { FC } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { useGetFolderQuery } from "../../Hooks/GraphQL";
import { usePlayback } from "../../Hooks/usePlayback";
import { usePlaylist } from "../../Hooks/usePlaylist";
import Folder from "./Folder";

const FolderWithData: FC = () => {
  const params = useParams();
  const { data, isLoading: loading } = useGetFolderQuery({ id: params.id! });
  const navigate = useNavigate();
  const { playPlaylist } = usePlayback();
  const { playlists, movePlaylistsToFolder } = usePlaylist();

  return (
    <Folder
      folder={
        data?.folder && {
          id: data.folder.id,
          name: data.folder.name,
          playlists: data.folder.playlists.map((playlist) => ({
            id: playlist.id,
            name: playlist.name,
            description: playlist.description,
          })),
        }
      }
      loading={loading}
      allPlaylists={playlists}
      onBack={() => navigate(-1)}
      onMovePlaylists={(playlistIds, folderId) =>
        movePlaylistsToFolder({ playlistIds, folderId })
      }
      onPlayPlaylist={(playlistId) =>
        playPlaylist({ playlistId, shuffle: false })
      }
    />
  );
};

export default FolderWithData;
