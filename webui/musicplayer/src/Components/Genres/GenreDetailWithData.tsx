import { keepPreviousData } from "@tanstack/react-query";
import { useMemo } from "react";
import { useParams } from "react-router-dom";
import {
  GetGenreTracksQuery,
  useInfiniteGetGenreTracksQuery,
} from "../../Hooks/GraphQL";
import { useLikes } from "../../Hooks/useLikes";
import { usePlayback } from "../../Hooks/usePlayback";
import { usePlaylist } from "../../Hooks/usePlaylist";
import { useTimeFormat } from "../../Hooks/useFormat";
import { usePlayTrack } from "../../Hooks/usePlayTrack";
import Tracks from "../Tracks/Tracks";

const PAGE_SIZE = 100;

/**
 * One genre's tracks.
 *
 * Reuses the Tracks list rather than growing a near-copy of it: a genre is a
 * filtered view of the library, and every row action means the same thing
 * here as there.
 */
const GenreDetailWithData = () => {
  const { id = "" } = useParams();
  const playTrack = usePlayTrack();
  const { formatTime } = useTimeFormat();
  const { playNext } = usePlayback();
  const { toggleLike } = useLikes();
  const { recentPlaylists, addTrackToPlaylist } = usePlaylist();
  const {
    data,
    isLoading: loading,
    hasNextPage,
    isFetchingNextPage,
    fetchNextPage,
  } = useInfiniteGetGenreTracksQuery(
    { id, limit: PAGE_SIZE },
    {
      initialPageParam: { offset: 0 },
      getNextPageParam: (
        lastPage: GetGenreTracksQuery,
        allPages: GetGenreTracksQuery[]
      ) =>
        lastPage.genreTracks.length === PAGE_SIZE
          ? { offset: allPages.length * PAGE_SIZE }
          : undefined,
      placeholderData: keepPreviousData,
      enabled: !!id,
    }
  );

  const tracks = useMemo(
    () =>
      (data?.pages.flatMap((page) => page.genreTracks) ?? []).map((track) => ({
        id: track.id,
        title: track.title,
        artist: track.artist,
        album: track.albumTitle ?? "",
        albumId: track.albumId ?? undefined,
        artistId: track.artistId ?? undefined,
        cover: track.cover ?? undefined,
        duration: formatTime((track.duration ?? 0) * 1000),
        key: track.key,
        bpm: track.bpm,
      })),
    [data, formatTime]
  );

  return (
    <Tracks
      tracks={tracks}
      loading={loading}
      filter=""
      recentPlaylists={recentPlaylists}
      onFilter={() => {}}
      onPlayTrack={(trackId) => playTrack(trackId)}
      onPlayNext={(trackId) => playNext({ trackId })}
      onToggleLike={(trackId) => toggleLike(trackId)}
      onAddTrackToPlaylist={(playlistId, trackId) =>
        addTrackToPlaylist({ playlistId, trackId })
      }
      onLoadMore={() => {
        if (hasNextPage && !isFetchingNextPage) {
          fetchNextPage({ cancelRefetch: false });
        }
      }}
      hasMore={!!hasNextPage}
      loadingMore={isFetchingNextPage}
    />
  );
};

export default GenreDetailWithData;
