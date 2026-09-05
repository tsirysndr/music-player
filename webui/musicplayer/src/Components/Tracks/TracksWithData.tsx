import { FC, useMemo, useState } from "react";
import { keepPreviousData } from "@tanstack/react-query";
import Tracks from "./Tracks";
import { GetTracksQuery, useInfiniteGetTracksQuery } from "../../Hooks/GraphQL";
import { useDevices } from "../../Hooks/useDevices";
import { useTimeFormat } from "../../Hooks/useFormat";
import { usePlayback } from "../../Hooks/usePlayback";
import { usePlaylist } from "../../Hooks/usePlaylist";
import { resourceUriResolver } from "../../ResourceUriResolver";

const PAGE_SIZE = 100;

const TracksWithData: FC = () => {
  const [filter, setFilter] = useState<string | undefined>(undefined);
  const {
    data,
    isLoading: loading,
    hasNextPage,
    isFetchingNextPage,
    fetchNextPage,
  } = useInfiniteGetTracksQuery(
    { filter, limit: PAGE_SIZE },
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
  const { currentCastDevice } = useDevices();
  const tracks = useMemo(
    () => (!loading && data ? data.pages.flatMap((page) => page.tracks) : []),
    [loading, data]
  );
  const { recentPlaylists, createPlaylist, addTrackToPlaylist } = usePlaylist();

  const onFilter = (filter: string) => {
    setFilter(filter.length > 0 ? filter : undefined);
  };

  const onLoadMore = () => {
    if (hasNextPage && !isFetchingNextPage) {
      fetchNextPage({ cancelRefetch: false });
    }
  };

  return (
    <Tracks
      tracks={tracks.map((track) => ({
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
      }))}
      nowPlaying={nowPlaying}
      onPlayTrack={(id, position) => {}}
      onPlayNext={(trackId) => playNext({ trackId })}
      onCreatePlaylist={(name, description) =>
        createPlaylist({ name, description })
      }
      onAddTrackToPlaylist={(playlistId, trackId) =>
        addTrackToPlaylist({ playlistId, trackId })
      }
      recentPlaylists={recentPlaylists}
      currentCastDevice={currentCastDevice}
      onFilter={onFilter}
      onLoadMore={onLoadMore}
      hasMore={!!hasNextPage}
    />
  );
};

export default TracksWithData;
