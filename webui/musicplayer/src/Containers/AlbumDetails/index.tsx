import { useEffect } from "react";
import { useNavigate, useParams } from "react-router-dom";
import AlbumDetails from "../../Components/AlbumDetails";
import { useGetAlbumQuery } from "../../Hooks/GraphQL";
import { useDevices } from "../../Hooks/useDevices";
import { useTimeFormat } from "../../Hooks/useFormat";
import { usePlayback } from "../../Hooks/usePlayback";
import { usePlaylist } from "../../Hooks/usePlaylist";
import { useSearch } from "../../Hooks/useSearch";

const AlbumDetailsPage = () => {
  const params = useParams();
  const { devices, currentDevice, connectToDevice, disconnectFromDevice } =
    useDevices();
  const { data, loading, refetch } = useGetAlbumQuery({
    variables: {
      id: params.id!,
    },
    fetchPolicy: "network-only",
  });

  useEffect(() => {
    params.id && refetch();
  }, [params.id, refetch]);

  const navigate = useNavigate();
  const { formatTime } = useTimeFormat();
  const {
    play,
    pause,
    next,
    previous,
    nowPlaying,
    nextTracks,
    previousTracks,
    playAlbum,
    playNext,
    playTrackAt,
    removeTrackAt,
    playPlaylist,
  } = usePlayback();
  const { onSearch } = useSearch();
  const album =
    !loading && data
      ? {
          ...data.album,
          tracks: data.album.tracks.map((track, index) => ({
            "#": track.trackNumber,
            id: track.id,
            title: track.title,
            artist: track.artists.map((artist) => artist.name).join(", "),
            time: formatTime(track.duration! * 1000),
            artistId: track.artists[0].id,
            albumId: data.album.id,
            index,
          })),
        }
      : { tracks: [] };
  const {
    folders,
    mainPlaylists,
    recentPlaylists,
    createFolder,
    createPlaylist,
    addTrackToPlaylist,
    deleteFolder,
    deletePlaylist,
    renameFolder,
    renamePlaylist,
  } = usePlaylist();
  return (
    <AlbumDetails
      onBack={() => navigate(-1)}
      onClickLibraryItem={(item) => navigate(`/${item}`)}
      onClickAlbum={() => {}}
      onPlay={() => play()}
      onPause={() => pause()}
      onNext={() => next()}
      onPrevious={() => previous()}
      onShuffle={() => {}}
      onRepeat={() => {}}
      album={album}
      nowPlaying={nowPlaying}
      nextTracks={nextTracks}
      previousTracks={previousTracks}
      onPlayAlbum={(albumId, shuffle, position) =>
        playAlbum({ variables: { albumId, position, shuffle } })
      }
      onPlayNext={(trackId) => playNext({ variables: { trackId } })}
      onPlayTrackAt={(position) => playTrackAt({ variables: { position } })}
      onRemoveTrackAt={(position) => removeTrackAt({ variables: { position } })}
      onSearch={(query) => navigate(`/search?q=${query}`)}
      folders={folders}
      playlists={mainPlaylists}
      onCreateFolder={(name) => createFolder({ variables: { name } })}
      onCreatePlaylist={(name) => createPlaylist({ variables: { name } })}
      onAddTrackToPlaylist={(playlistId, trackId) =>
        addTrackToPlaylist({ variables: { playlistId, trackId } })
      }
      onDeleteFolder={(id) => deleteFolder({ variables: { id } })}
      onDeletePlaylist={(id) => deletePlaylist({ variables: { id } })}
      onEditFolder={(id, name) => renameFolder({ variables: { id, name } })}
      onEditPlaylist={(id, name) => renamePlaylist({ variables: { id, name } })}
      onPlayPlaylist={(playlistId, shuffle, position) =>
        playPlaylist({ variables: { playlistId, position, shuffle } })
      }
      recentPlaylists={recentPlaylists}
      devices={devices}
      currentDevice={currentDevice}
      connectToDevice={(id) => connectToDevice({ variables: { id } })}
      disconnectFromDevice={() => disconnectFromDevice()}
    />
  );
};

export default AlbumDetailsPage;
