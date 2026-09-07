import { keepPreviousData } from "@tanstack/react-query";
import { FC, useMemo, useState } from "react";
import {
  GetArtistsQuery,
  useInfiniteGetArtistsQuery,
} from "../../Hooks/GraphQL";
import { usePlayback } from "../../Hooks/usePlayback";
import Artists from "./Artists";

const PAGE_SIZE = 100;

const ArtistsWithData: FC = () => {
  const [filter, setFilter] = useState("");
  const {
    data,
    isLoading: loading,
    hasNextPage,
    isFetchingNextPage,
    fetchNextPage,
  } = useInfiniteGetArtistsQuery(
    { filter: filter || undefined, limit: PAGE_SIZE },
    {
      initialPageParam: { offset: 0 },
      getNextPageParam: (
        lastPage: GetArtistsQuery,
        allPages: GetArtistsQuery[]
      ) =>
        lastPage.artists.length === PAGE_SIZE
          ? { offset: allPages.length * PAGE_SIZE }
          : undefined,
      placeholderData: keepPreviousData,
    }
  );
  const { playArtistTracks } = usePlayback();

  const artists = useMemo(
    () =>
      (data?.pages.flatMap((page) => page.artists) ?? []).map((artist) => ({
        id: artist.id,
        name: artist.name,
        picture: artist.picture,
      })),
    [data]
  );

  return (
    <Artists
      artists={artists}
      loading={loading}
      filter={filter}
      onFilter={setFilter}
      onPlayArtist={(artistId) =>
        playArtistTracks({ artistId, shuffle: false })
      }
      onLoadMore={() => {
        if (hasNextPage && !isFetchingNextPage) {
          fetchNextPage({ cancelRefetch: false });
        }
      }}
      hasMore={!!hasNextPage}
    />
  );
};

export default ArtistsWithData;
