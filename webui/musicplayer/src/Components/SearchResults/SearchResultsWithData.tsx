import { FC, useEffect } from "react";
import { useSearchParams } from "react-router-dom";
import { useTimeFormat } from "../../Hooks/useFormat";
import { useLikes } from "../../Hooks/useLikes";
import { usePlayback } from "../../Hooks/usePlayback";
import { usePlaylist } from "../../Hooks/usePlaylist";
import { usePlayTrack } from "../../Hooks/usePlayTrack";
import { useSearch } from "../../Hooks/useSearch";
import SearchResults from "./SearchResults";

const SearchResultsWithData: FC = () => {
  const { formatTime } = useTimeFormat();
  const { nowPlaying, playNext, playAlbum, playArtistTracks } = usePlayback();
  const playTrack = usePlayTrack();
  const { isLiked, toggleLike } = useLikes();
  const { recentPlaylists, addTrackToPlaylist } = usePlaylist();
  const [params, setParams] = useSearchParams();
  const { onSearch, query, results } = useSearch();

  // The query lives in the url, so a search survives a reload and can be
  // shared; the atom follows it rather than the other way round.
  const q = params.get("q") ?? "";
  useEffect(() => {
    if (q !== query) onSearch(q);
    // `onSearch` is a fresh closure on every render, so depending on it here
    // would re-run the search forever.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [q]);

  return (
    <SearchResults
      query={q}
      tracks={results.tracks.map((track) => ({
        id: track.id,
        title: track.title,
        artist: track.artist,
        artistId: track.artistId,
        album: track.album,
        albumId: track.albumId,
        duration: formatTime((track.duration ?? 0) * 1000),
        liked: isLiked(track.id),
      }))}
      albums={results.albums.map((album) => ({
        id: album.id,
        title: album.title,
        artist: album.artist,
        cover: album.cover,
      }))}
      artists={results.artists.map((artist) => ({
        id: artist.id,
        name: artist.name,
        picture: artist.picture,
      }))}
      currentTrackId={nowPlaying?.isPlaying ? nowPlaying.id : undefined}
      recentPlaylists={recentPlaylists}
      onSearch={(next) =>
        setParams(next ? { q: next } : {}, { replace: true })
      }
      onPlayTrack={(id) => playTrack(id)}
      onPlayNext={(trackId) => playNext({ trackId })}
      onToggleLike={toggleLike}
      onAddTrackToPlaylist={(playlistId, trackId) =>
        addTrackToPlaylist({ playlistId, trackId })
      }
      onPlayAlbum={(albumId, shuffle) => playAlbum({ albumId, shuffle })}
      onPlayArtist={(artistId) => playArtistTracks({ artistId, shuffle: false })}
    />
  );
};

export default SearchResultsWithData;
