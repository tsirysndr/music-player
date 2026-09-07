import { createStore } from "jotai";
import { describe, expect, it } from "vitest";
import { searchQueryAtom, searchResultsAtom } from "./search";

describe("search atoms", () => {
  it("start with an empty query and no results", () => {
    const store = createStore();
    expect(store.get(searchQueryAtom)).toBe("");
    expect(store.get(searchResultsAtom)).toEqual({
      tracks: [],
      artists: [],
      albums: [],
      playlists: [],
    });
  });

  it("holds the results of a search", () => {
    const store = createStore();
    store.set(searchQueryAtom, "sabbath");
    store.set(searchResultsAtom, {
      tracks: [],
      artists: [
        {
          id: "50fcd83c68216360292343903c891320",
          name: "Black Sabbath",
          picture: "",
        },
      ],
      albums: [],
      playlists: [],
    });

    expect(store.get(searchQueryAtom)).toBe("sabbath");
    expect(store.get(searchResultsAtom).artists[0].name).toBe("Black Sabbath");
  });

  /**
   * A new search replaces the whole result set rather than merging into it —
   * otherwise the previous query's albums stay on screen under the new one's
   * tracks.
   */
  it("replaces the previous results rather than merging", () => {
    const store = createStore();
    store.set(searchResultsAtom, {
      tracks: [],
      artists: [{ id: "1", name: "Black Sabbath", picture: "" }],
      albums: [{ id: "a", title: "13", artist: "Black Sabbath", year: 2013 }],
      playlists: [],
    });

    store.set(searchResultsAtom, {
      tracks: [],
      artists: [],
      albums: [],
      playlists: [],
    });

    expect(store.get(searchResultsAtom).artists).toEqual([]);
    expect(store.get(searchResultsAtom).albums).toEqual([]);
  });
});
