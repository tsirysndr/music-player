import { useEffect } from "react";
import { useNavigate, useParams } from "react-router-dom";
import ArtistDetails from "../../Components/ArtistDetails";
import { useGetArtistQuery } from "../../Hooks/GraphQL";
import { useDevices } from "../../Hooks/useDevices";
import { useTimeFormat } from "../../Hooks/useFormat";
import { usePlayback } from "../../Hooks/usePlayback";
import { usePlaylist } from "../../Hooks/usePlaylist";
import { useSearch } from "../../Hooks/useSearch";
import { resourceUriResolver } from "../../ResourceUriResolver";

const ArtistDetailsPage = () => {
  const params = useParams();
  const { data, loading, refetch } = useGetArtistQuery({
    variables: {
      id: params.id!,
    },
    fetchPolicy: "network-only",
  });

  useEffect(() => {
    params.id && refetch();
  }, [params.id, refetch]);

  const { formatTime } = useTimeFormat();
  const navigate = useNavigate();
  const {
    play,
    pause,
    next,
    previous,
    nowPlaying,
    nextTracks,
    previousTracks,
    playArtistTracks,
    playNext,
    playTrackAt,
    removeTrackAt,
    playPlaylist,
  } = usePlayback();
  const { onSearch } = useSearch();
  const {
    devices,
    castDevices,
    currentDevice,
    currentCastDevice,
    connectToDevice,
    disconnectFromDevice,
    connectToCastDevice,
    disconnectFromCastDevice,
  } = useDevices();
  const artist = !loading && data ? data.artist : {};
  const tracks =
    !loading && data
      ? data.artist.songs.map((track, index) => ({
          id: track.id,
          title: track.title,
          artist: track.artist,
          album: track.album.title,
          time: formatTime(track.duration! * 1000),
          cover: track.album.cover
            ? resourceUriResolver.resolve(`/covers/${track.album.cover}`)
            : undefined,
          artistId: track.artists[0].id,
          albumId: track.album.id,
          index,
        }))
      : [];
  const albums =
    !loading && data
      ? data.artist.albums.map((album) => ({
          id: album.id,
          title: album.title,
          artist: album.artist,
          cover: album.cover
            ? resourceUriResolver.resolve(`/covers/${album.cover}`)
            : undefined,
        }))
      : [];
  const {
    folders,
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
    <ArtistDetails
      onBack={() => navigate(-1)}
      onClickLibraryItem={(item) => navigate(`/${item}`)}
      onPlay={() => play()}
      onPause={() => pause()}
      onNext={() => next()}
      onPrevious={() => previous()}
      onShuffle={() => {}}
      onRepeat={() => {}}
      artist={artist}
      tracks={tracks}
      albums={albums}
      nowPlaying={nowPlaying}
      nextTracks={nextTracks}
      previousTracks={previousTracks}
      onPlayArtistTracks={(artistId, shuffle, position) =>
        playArtistTracks({ variables: { artistId, position, shuffle } })
      }
      onPlayNext={(trackId) => playNext({ variables: { trackId } })}
      onPlayTrackAt={(position) => playTrackAt({ variables: { position } })}
      onRemoveTrackAt={(position) => removeTrackAt({ variables: { position } })}
      onSearch={(query) => navigate(`/search?q=${query}`)}
      folders={folders}
      playlists={mainPlaylists}
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
      onEditPlaylist={(id, name) => renamePlaylist({ variables: { id, name } })}
      onPlayPlaylist={(playlistId, shuffle, position) =>
        playPlaylist({ variables: { playlistId, position, shuffle } })
      }
      recentPlaylists={recentPlaylists}
      devices={devices}
      castDevices={castDevices}
      currentDevice={currentDevice}
      currentCastDevice={currentCastDevice}
      connectToDevice={(id) => connectToDevice({ variables: { id } })}
      disconnectFromDevice={() => disconnectFromDevice()}
      connectToCastDevice={(id) => connectToCastDevice({ variables: { id } })}
      disconnectFromCastDevice={() => disconnectFromCastDevice()}
    />
  );
};

export default ArtistDetailsPage;
