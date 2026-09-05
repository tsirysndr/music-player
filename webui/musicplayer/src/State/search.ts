import { atom } from "jotai";
import { Album, Artist, Track } from "../Types";

export type SearchResults = {
  tracks: Track[];
  artists: Artist[];
  albums: Album[];
  playlists: any[];
};

export const searchQueryAtom = atom<string>("");

export const searchResultsAtom = atom<SearchResults>({
  tracks: [],
  artists: [],
  albums: [],
  playlists: [],
});
