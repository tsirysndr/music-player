import { useState } from "react";
import { useSearchLazyQuery } from "./GraphQL";
import { Album, Artist, Track } from "../Types";

export const useSearch = () => {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<{
    tracks: Track[];
    artists: Artist[];
    albums: Album[];
    playlists: any[];
  }>({
    albums: [],
    artists: [],
    tracks: [],
    playlists: [],
  });

  const [search] = useSearchLazyQuery();

  const onSearch = async (keyword: string) => {
    setQuery(query);
    const { data } = await search({
      variables: {
        keyword,
      },
    });

    setResults({
      albums:
        data?.search.albums.map((x) => ({
          ...x,
          cover: x.cover ? `/covers/${x.cover}` : undefined,
          year: 0,
        })) || [],
      artists: data?.search.artists || [],
      tracks:
        data?.search.tracks.map((x) => ({
          ...x,
          cover: x.cover ? `/covers/${x.cover}` : undefined,
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
