import { keepPreviousData } from "@tanstack/react-query";
import { FC, useMemo, useState } from "react";
import { GetAlbumsQuery, useInfiniteGetAlbumsQuery } from "../../Hooks/GraphQL";
import { usePlayback } from "../../Hooks/usePlayback";
import Albums from "./Albums";

const PAGE_SIZE = 100;

const AlbumsWithData: FC = () => {
  const [filter, setFilter] = useState("");
  const {
    data,
    isLoading: loading,
    hasNextPage,
    isFetchingNextPage,
    fetchNextPage,
  } = useInfiniteGetAlbumsQuery(
    { filter: filter || undefined, limit: PAGE_SIZE },
    {
      initialPageParam: { offset: 0 },
      getNextPageParam: (lastPage: GetAlbumsQuery, allPages: GetAlbumsQuery[]) =>
        lastPage.albums.length === PAGE_SIZE
          ? { offset: allPages.length * PAGE_SIZE }
          : undefined,
      placeholderData: keepPreviousData,
    }
  );
  const { playAlbum } = usePlayback();

  const albums = useMemo(() => {
    // Some remote sources can return the same album more than once. Keep one
    // card per stable album id while combining paginated results.
    const unique = new Map<string, GetAlbumsQuery["albums"][number]>();
    for (const album of data?.pages.flatMap((page) => page.albums) ?? []) {
      if (!unique.has(album.id)) unique.set(album.id, album);
    }
    return Array.from(unique.values()).map((album) => ({
      id: album.id,
      title: album.title,
      artist: album.artist,
      year: album.year,
      cover: album.cover ? `/covers/${album.cover}` : undefined,
    }));
  }, [data]);

  return (
    <Albums
      albums={albums}
      loading={loading}
      filter={filter}
      onFilter={setFilter}
      onPlayAlbum={(albumId) => playAlbum({ albumId, shuffle: false })}
      onShuffleAlbum={(albumId) => playAlbum({ albumId, shuffle: true })}
      onLoadMore={() => {
        if (hasNextPage && !isFetchingNextPage) {
          fetchNextPage({ cancelRefetch: false });
        }
      }}
      hasMore={!!hasNextPage}
    />
  );
};

export default AlbumsWithData;
