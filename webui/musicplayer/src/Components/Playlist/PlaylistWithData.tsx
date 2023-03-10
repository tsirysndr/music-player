import { FC } from "react";
import Playlist from "./Playlist";
import { useGetPlaylistQuery } from "../../Hooks/GraphQL";
import { useDevices } from "../../Hooks/useDevices";
import { usePlayback } from "../../Hooks/usePlayback";
import { usePlaylist } from "../../Hooks/usePlaylist";
import { useNavigate, useParams } from "react-router-dom";

const PlaylistWithData: FC = () => {
  const params = useParams();
  const { data } = useGetPlaylistQuery({
    variables: {
      id: params.id!,
    },
    fetchPolicy: "network-only",
  });
  const navigate = useNavigate();
  const { currentCastDevice } = useDevices();
  const { nowPlaying, playNext, playPlaylist } = usePlayback();

  const { recentPlaylists, createPlaylist, addTrackToPlaylist } = usePlaylist();
  return (
    <Playlist
      onBack={() => navigate(-1)}
      nowPlaying={nowPlaying}
      onPlayNext={(trackId) => playNext({ variables: { trackId } })}
      onCreatePlaylist={(name, description) =>
        createPlaylist({ variables: { name, description } })
      }
      onAddTrackToPlaylist={(playlistId, trackId) =>
        addTrackToPlaylist({ variables: { trackId, playlistId } })
      }
      onPlayPlaylist={(playlistId, shuffle, position) =>
        playPlaylist({ variables: { playlistId, position, shuffle } })
      }
      playlist={data?.playlist}
      recentPlaylists={recentPlaylists}
      currentCastDevice={currentCastDevice}
    />
  );
};

export default PlaylistWithData;
