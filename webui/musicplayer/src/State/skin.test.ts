import { createStore } from "jotai";
import { beforeEach, describe, expect, it } from "vitest";
import {
  DEFAULT_SKIN,
  isSkinId,
  SKINS,
  SKIN_STORAGE_KEY,
  skinAtom,
  storedSkin,
} from "./skin";

describe("SKINS", () => {
  /**
   * The ids key the `[data-skin="…"]` blocks in `styles/skins.css`, and the
   * names are what both clients display. A skin listed here with no block
   * there renders in the default palette with no error, so the two lists have
   * to be checked against each other.
   */
  it("lists the five skins the desktop ships, in cycle order", () => {
    expect(SKINS.map((skin) => skin.id)).toEqual([
      "synthwave",
      "late-night",
      "neutron",
      "lunar",
      "porcelain",
    ]);
    expect(SKINS.map((skin) => skin.name)).toEqual([
      "Synthwave",
      "Late Night",
      "Neutron",
      "Lunar",
      "Porcelain",
    ]);
  });

  it("defaults to the same skin the desktop defaults to", () => {
    expect(DEFAULT_SKIN).toBe("late-night");
    expect(isSkinId(DEFAULT_SKIN)).toBe(true);
  });
});

describe("isSkinId", () => {
  it("accepts every shipped id", () => {
    expect(SKINS.every((skin) => isSkinId(skin.id))).toBe(true);
  });

  it("rejects anything else", () => {
    for (const value of ["", "dark", "Synthwave", null, undefined, 3, {}]) {
      expect(isSkinId(value)).toBe(false);
    }
  });
});

describe("storedSkin", () => {
  beforeEach(() => localStorage.clear());

  it("falls back to the default with nothing saved", () => {
    expect(storedSkin()).toBe(DEFAULT_SKIN);
  });

  it("returns a saved skin", () => {
    localStorage.setItem(SKIN_STORAGE_KEY, "synthwave");
    expect(storedSkin()).toBe("synthwave");
  });

  /**
   * localStorage is user-writable and survives a release that drops a skin, so
   * a value that is no longer an id has to fall back rather than be trusted
   * onto `data-skin`.
   */
  it("falls back when the saved value is not a skin any more", () => {
    localStorage.setItem(SKIN_STORAGE_KEY, "midnight-purple");
    expect(storedSkin()).toBe(DEFAULT_SKIN);
  });
});

describe("skinAtom", () => {
  it("holds a skin and can be cycled through the list", () => {
    const store = createStore();
    store.set(skinAtom, "synthwave");
    expect(store.get(skinAtom)).toBe("synthwave");

    // What `cycleSkin` does: step to the next entry, wrapping at the end.
    const next = (current: string) => {
      const index = SKINS.findIndex((skin) => skin.id === current);
      return SKINS[(index + 1) % SKINS.length].id;
    };

    store.set(skinAtom, next(store.get(skinAtom)));
    expect(store.get(skinAtom)).toBe("late-night");

    store.set(skinAtom, "porcelain");
    store.set(skinAtom, next(store.get(skinAtom)));
    expect(store.get(skinAtom)).toBe("synthwave");
  });
});
