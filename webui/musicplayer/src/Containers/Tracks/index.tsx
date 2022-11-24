import { useNavigate } from "react-router-dom";
import Tracks from "../../Components/Tracks";
import { useGetTracksQuery } from "../../Hooks/GraphQL";
import { useTimeFormat } from "../../Hooks/useFormat";
import { usePlaylist } from "../../Hooks/usePlaylist";
import { useSearch } from "../../Hooks/useSearch";

const TracksPage = () => {
  const { data, loading } = useGetTracksQuery({
    fetchPolicy: "cache-and-network",
  });
  const navigate = useNavigate();
  const { formatTime } = useTimeFormat();
  const { onSearch } = useSearch();
  const tracks = !loading && data ? data.tracks : [];
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
    <>
      <Tracks
        tracks={tracks.slice(0, 100).map((track) => ({
          id: track.id,
          title: track.title,
          artist: track.artist,
          album: track.album.title,
          time: formatTime(track.duration! * 1000),
          cover: track.album.cover ? `/covers/${track.album.cover}` : undefined,
          artistId: track.artists[0].id,
          albumId: track.album.id,
        }))}
        onClickLibraryItem={(item) => navigate(`/${item}`)}
        onPlayTrack={(id, position) => {}}
        onSearch={(query) => navigate(`/search?q=${query}`)}
        playlists={mainPlaylists}
        folders={folders}
        onCreateFolder={(name) => createFolder({ variables: { name } })}
        onCreatePlaylist={(name, description) =>
          createPlaylist({ variables: { name, description } })
        }
        onAddTrackToPlaylist={(playlistId, trackId) =>
          addTrackToPlaylist({ variables: { playlistId, trackId } })
        }
        onDeleteFolder={(id) => deleteFolder({ variables: { id } })}
        onDeletePlaylist={(id) => deletePlaylist({ variables: { id } })}
        onEditFolder={(id, name) => renameFolder({ variables: { id, name } })}
        onEditPlaylist={(id, name, description) =>
          renamePlaylist({ variables: { id, name } })
        }
        recentPlaylists={recentPlaylists}
      />
    </>
  );
};

export default TracksPage;
