import { FC, useMemo, useState } from "react";
import { keepPreviousData } from "@tanstack/react-query";
import { useNavigate } from "react-router-dom";
import Artists from "./Artists";
import {
  GetArtistsQuery,
  useInfiniteGetArtistsQuery,
} from "../../Hooks/GraphQL";
import { useDevices } from "../../Hooks/useDevices";

const PAGE_SIZE = 100;

const ArtistsWithData: FC = () => {
  const [filter, setFilter] = useState<string | undefined>(undefined);
  const {
    data,
    isLoading: loading,
    hasNextPage,
    isFetchingNextPage,
    fetchNextPage,
  } = useInfiniteGetArtistsQuery(
    { filter, limit: PAGE_SIZE },
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
  const navigate = useNavigate();
  const { currentCastDevice } = useDevices();
  const artists = useMemo(
    () => (!loading && data ? data.pages.flatMap((page) => page.artists) : []),
    [loading, data]
  );
  const onFilter = (filter: string) => {
    setFilter(filter.length > 0 ? filter : undefined);
  };
  const onLoadMore = () => {
    if (hasNextPage && !isFetchingNextPage) {
      fetchNextPage({ cancelRefetch: false });
    }
  };
  return (
    <Artists
      artists={artists.map((artist) => ({
        id: artist.id,
        name: artist.name,
        cover: artist.picture,
      }))}
      onClickArtist={({ id }) => navigate(`/artists/${id}`)}
      currentCastDevice={currentCastDevice}
      onFilter={onFilter}
      onLoadMore={onLoadMore}
      hasMore={!!hasNextPage}
    />
  );
};

export default ArtistsWithData;
