import { keepPreviousData } from "@tanstack/react-query";
import { FC, useMemo, useState } from "react";
import { GetTracksQuery, useInfiniteGetTracksQuery } from "../../Hooks/GraphQL";
import { useTimeFormat } from "../../Hooks/useFormat";
import { useLikes } from "../../Hooks/useLikes";
import { usePlayback } from "../../Hooks/usePlayback";
import { usePlaylist } from "../../Hooks/usePlaylist";
import { usePlayTrack } from "../../Hooks/usePlayTrack";
import Tracks from "./Tracks";

const PAGE_SIZE = 100;

const TracksWithData: FC = () => {
  const [filter, setFilter] = useState("");
  const {
    data,
    isLoading: loading,
    hasNextPage,
    isFetchingNextPage,
    fetchNextPage,
  } = useInfiniteGetTracksQuery(
    { filter: filter || undefined, limit: PAGE_SIZE },
    {
      initialPageParam: { offset: 0 },
      getNextPageParam: (lastPage: GetTracksQuery, allPages: GetTracksQuery[]) =>
        lastPage.tracks.length === PAGE_SIZE
          ? { offset: allPages.length * PAGE_SIZE }
          : undefined,
      placeholderData: keepPreviousData,
    }
  );
  const { formatTime } = useTimeFormat();
  const { nowPlaying, playNext } = usePlayback();
  const playTrack = usePlayTrack();
  const { isLiked, toggleLike } = useLikes();
  const { recentPlaylists, addTrackToPlaylist } = usePlaylist();

  const tracks = useMemo(
    () =>
      (data?.pages.flatMap((page) => page.tracks) ?? []).map((track) => ({
        id: track.id,
        title: track.title,
        artist: track.artist,
        artistId: track.artists[0]?.id,
        album: track.album.title,
        albumId: track.album.id,
        duration: formatTime((track.duration ?? 0) * 1000),
        key: track.key,
        bpm: track.bpm,
        liked: isLiked(track.id),
      })),
    [data, formatTime, isLiked]
  );

  return (
    <Tracks
      tracks={tracks}
      loading={loading}
      currentTrackId={nowPlaying?.isPlaying ? nowPlaying.id : undefined}
      filter={filter}
      recentPlaylists={recentPlaylists}
      onFilter={setFilter}
      onPlayTrack={(id) => playTrack(id)}
      onPlayNext={(trackId) => playNext({ trackId })}
      onToggleLike={toggleLike}
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

export default TracksWithData;
