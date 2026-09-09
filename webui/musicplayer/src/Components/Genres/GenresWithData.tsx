import { keepPreviousData } from "@tanstack/react-query";
import { useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import { GetGenresQuery, useInfiniteGetGenresQuery } from "../../Hooks/GraphQL";
import Genres from "./Genres";

const PAGE_SIZE = 100;

const GenresWithData = () => {
  const navigate = useNavigate();
  const [filter, setFilter] = useState("");
  const {
    data,
    isLoading: loading,
    hasNextPage,
    isFetchingNextPage,
    fetchNextPage,
  } = useInfiniteGetGenresQuery(
    { limit: PAGE_SIZE },
    {
      initialPageParam: { offset: 0 },
      getNextPageParam: (lastPage: GetGenresQuery, allPages: GetGenresQuery[]) =>
        lastPage.genres.length === PAGE_SIZE
          ? { offset: allPages.length * PAGE_SIZE }
          : undefined,
      placeholderData: keepPreviousData,
    }
  );

  const genres = useMemo(() => {
    const needle = filter.trim().toLowerCase();
    // Filtered here rather than by re-querying: the list is short, already
    // loaded, and a round trip per keystroke would be slower than the typing.
    return (data?.pages.flatMap((page) => page.genres) ?? [])
      .filter((genre) => !needle || genre.name.toLowerCase().includes(needle))
      .map((genre) => ({
        id: genre.id,
        name: genre.name,
        trackCount: genre.trackCount,
      }));
  }, [data, filter]);

  return (
    <Genres
      genres={genres}
      loading={loading}
      filter={filter}
      onFilter={setFilter}
      onOpenGenre={(id) => navigate(`/genres/${id}`)}
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

export default GenresWithData;
