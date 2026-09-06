import { FC, useMemo, useState } from "react";
import { keepPreviousData } from "@tanstack/react-query";
import Albums from "./Albums";
import { GetAlbumsQuery, useInfiniteGetAlbumsQuery } from "../../Hooks/GraphQL";
import { useDevices } from "../../Hooks/useDevices";
import { resourceUriResolver } from "../../ResourceUriResolver";
import { useNavigate } from "react-router-dom";

const PAGE_SIZE = 100;

const AlbumsWithData: FC = () => {
  const [filter, setFilter] = useState<string | undefined>(undefined);
  const {
    data,
    isLoading: loading,
    hasNextPage,
    isFetchingNextPage,
    fetchNextPage,
  } = useInfiniteGetAlbumsQuery(
    { filter, limit: PAGE_SIZE },
    {
      initialPageParam: { offset: 0 },
      getNextPageParam: (lastPage: GetAlbumsQuery, allPages: GetAlbumsQuery[]) =>
        lastPage.albums.length === PAGE_SIZE
          ? { offset: allPages.length * PAGE_SIZE }
          : undefined,
      placeholderData: keepPreviousData,
    }
  );
  const navigate = useNavigate();
  const { currentCastDevice } = useDevices();
  const albums = useMemo(
    () => {
      if (loading || !data) return [];
      // Some remote sources can return the same album more than once. Keep
      // one card per stable album id while combining paginated results.
      const unique = new Map<string, GetAlbumsQuery["albums"][number]>();
      for (const album of data.pages.flatMap((page) => page.albums)) {
        if (!unique.has(album.id)) unique.set(album.id, album);
      }
      return Array.from(unique.values());
    },
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
    <Albums
      albums={albums.map((album) => ({
        id: album.id,
        title: album.title,
        artist: album.artist,
        cover:
          album.cover && resourceUriResolver.resolve(`/covers/${album.cover}`),
      }))}
      onClickAlbum={({ id }) => navigate(`/albums/${id}`)}
      currentCastDevice={currentCastDevice}
      onFilter={onFilter}
      onLoadMore={onLoadMore}
      hasMore={!!hasNextPage}
    />
  );
};

export default AlbumsWithData;
