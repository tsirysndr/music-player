import { useAtom } from "jotai";
import { useQueryClient } from "@tanstack/react-query";
import { useSearchQuery } from "./GraphQL";
import { searchQueryAtom, searchResultsAtom } from "../State";
import { resourceUriResolver } from "../ResourceUriResolver";

export const useSearch = () => {
  const [query, setQuery] = useAtom(searchQueryAtom);
  const [results, setResults] = useAtom(searchResultsAtom);
  const queryClient = useQueryClient();

  const onSearch = async (keyword: string) => {
    setQuery(keyword);
    const data = await queryClient.fetchQuery({
      queryKey: useSearchQuery.getKey({ keyword }),
      queryFn: useSearchQuery.fetcher({ keyword }),
    });

    setResults({
      albums:
        data?.search.albums.map((x) => ({
          ...x,
          cover: x.cover
            ? resourceUriResolver.resolve(`/covers/${x.cover}`)
            : undefined,
          year: 0,
        })) || [],
      artists: data?.search.artists || [],
      tracks:
        data?.search.tracks.map((x) => ({
          ...x,
          cover: x.cover
            ? resourceUriResolver.resolve(`/covers/${x.cover}`)
            : undefined,
          duration: x.duration!,
          album: x.albumTitle,
          artistId: x.artistId,
          albumId: x.albumId,
        })) || [],
      playlists: [],
    });
  };

  return {
    query,
    results,
    onSearch,
  };
};
