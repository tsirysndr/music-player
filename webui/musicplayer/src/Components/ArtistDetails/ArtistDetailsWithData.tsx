import { FC, useEffect } from "react";
import { useNavigate, useParams } from "react-router-dom";
import ArtistDetails from "./ArtistDetails";
import { useGetArtistQuery } from "../../Hooks/GraphQL";
import { useDevices } from "../../Hooks/useDevices";
import { useTimeFormat } from "../../Hooks/useFormat";
import { usePlayback } from "../../Hooks/usePlayback";
import { usePlaylist } from "../../Hooks/usePlaylist";
import { resourceUriResolver } from "../../ResourceUriResolver";

const ArtistDetailsWithData: FC = () => {
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
  const { playArtistTracks, playNext } = usePlayback();
  const { currentCastDevice } = useDevices();
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
  const { recentPlaylists, createPlaylist, addTrackToPlaylist } = usePlaylist();
  return (
    <ArtistDetails
      onBack={() => navigate(-1)}
      artist={artist}
      tracks={tracks}
      albums={albums}
      onPlayArtistTracks={(artistId, shuffle, position) =>
        playArtistTracks({ variables: { artistId, position, shuffle } })
      }
      onPlayNext={(trackId) => playNext({ variables: { trackId } })}
      onCreatePlaylist={(name, description) =>
        createPlaylist({ variables: { name, description } })
      }
      onAddTrackToPlaylist={(playlistId, trackId) =>
        addTrackToPlaylist({ variables: { playlistId, trackId } })
      }
      recentPlaylists={recentPlaylists}
      currentCastDevice={currentCastDevice}
    />
  );
};

export default ArtistDetailsWithData;
