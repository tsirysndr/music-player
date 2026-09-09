import { describe, expect, it } from "vitest";
import { isActive, NAV, PRIMARY_TABS, titleForPath } from "./navigation";

const entry = (to: string) => NAV.find((candidate) => candidate.to === to)!;

describe("NAV", () => {
  it("lists the library sections in the desktop's order", () => {
    expect(NAV.map((section) => section.to)).toEqual([
      "/albums",
      "/artists",
      "/genres",
      "/tracks",
      "/liked",
      "/playlists",
      "/radio",
      "/extensions",
      "/servers",
    ]);
  });

  /** Every primary tab has to be a real section, or a tab links nowhere. */
  it("draws the phone's primary tabs from the same list", () => {
    for (const path of PRIMARY_TABS) {
      expect(NAV.some((section) => section.to === path)).toBe(true);
    }
    // Four tabs plus "More" is what fits across a phone.
    expect(PRIMARY_TABS).toHaveLength(4);
  });
});

describe("isActive", () => {
  it("lights a section on its own route", () => {
    expect(isActive("/albums", entry("/albums"))).toBe(true);
    expect(isActive("/radio", entry("/radio"))).toBe(true);
  });

  it("keeps a section lit on its detail pages", () => {
    expect(isActive("/albums/3b16b0ca", entry("/albums"))).toBe(true);
    expect(isActive("/playlists/pl-1", entry("/playlists"))).toBe(true);
  });

  /** `/` is the tracks page, so Tracks is what should be lit at the root. */
  it("treats the root as Tracks", () => {
    expect(isActive("/", entry("/tracks"))).toBe(true);
    expect(isActive("/tracks", entry("/tracks"))).toBe(true);
  });

  /**
   * Only Tracks claims the root. Without the special case, every section's
   * `matches` containing "/" would light all of them at once.
   */
  it("does not light every section at the root", () => {
    const lit = NAV.filter((section) => isActive("/", section));
    expect(lit.map((section) => section.to)).toEqual(["/tracks"]);
  });

  it("lights Playlists on a folder page", () => {
    expect(isActive("/folders/folder-moods", entry("/playlists"))).toBe(true);
  });

  it("does not light a section on someone else's route", () => {
    expect(isActive("/artists", entry("/albums"))).toBe(false);
    expect(isActive("/search", entry("/tracks"))).toBe(false);
  });
});

describe("titleForPath", () => {
  it("names the section", () => {
    expect(titleForPath("/albums")).toBe("Albums");
    expect(titleForPath("/extensions")).toBe("Extensions");
    expect(titleForPath("/radio")).toBe("Internet Radio");
    expect(titleForPath("/liked")).toBe("Liked tracks");
  });

  it("names the record type on a detail page", () => {
    expect(titleForPath("/albums/3b16b0ca")).toBe("Album");
    expect(titleForPath("/artists/50fcd83c")).toBe("Artist");
    expect(titleForPath("/playlists/pl-1")).toBe("Playlist");
    expect(titleForPath("/folders/f-1")).toBe("Folder");
  });

  it("names the tracks page at the root", () => {
    expect(titleForPath("/")).toBe("Tracks");
  });

  it("falls back to the app name on a route it does not know", () => {
    expect(titleForPath("/nowhere")).toBe("Music Player");
  });
});
