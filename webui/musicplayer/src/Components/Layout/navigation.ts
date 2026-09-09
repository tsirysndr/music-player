import { Icons, type IconComponent } from "../UI";

export type NavEntry = {
  to: string;
  label: string;
  icon: IconComponent;
  /** Extra path prefixes that should light this entry up. */
  matches?: string[];
};

/** The library sections, in the order the desktop sidebar lists them. */
export const NAV: NavEntry[] = [
  { to: "/albums", label: "Albums", icon: Icons.disc },
  { to: "/artists", label: "Artists", icon: Icons.artist },
  { to: "/genres", label: "Genres", icon: Icons.genre },
  { to: "/tracks", label: "Tracks", icon: Icons.listMusic, matches: ["/"] },
  { to: "/liked", label: "Liked", icon: Icons.heart },
  {
    to: "/playlists",
    label: "Playlists",
    icon: Icons.playlist,
    matches: ["/folders"],
  },
  { to: "/radio", label: "Internet Radio", icon: Icons.broadcast },
  { to: "/extensions", label: "Extensions", icon: Icons.extension },
  { to: "/servers", label: "Servers", icon: Icons.server },
];

/** The four that get their own tab on a phone; the rest live in "More". */
export const PRIMARY_TABS = ["/tracks", "/albums", "/artists", "/playlists"];

export const isActive = (pathname: string, entry: NavEntry) => {
  if (entry.to === "/tracks") {
    return pathname === "/" || pathname.startsWith("/tracks");
  }
  return (
    pathname.startsWith(entry.to) ||
    (entry.matches ?? []).some(
      (prefix) => prefix !== "/" && pathname.startsWith(prefix)
    )
  );
};

/** The heading the header shows for a route, mirroring the desktop's titles. */
export const titleForPath = (pathname: string) => {
  if (pathname.startsWith("/albums/")) return "Album";
  if (pathname.startsWith("/artists/")) return "Artist";
  if (pathname.startsWith("/playlists/")) return "Playlist";
  if (pathname.startsWith("/folders/")) return "Folder";
  if (pathname.startsWith("/search")) return "Search";
  if (pathname.startsWith("/liked")) return "Liked tracks";
  if (pathname.startsWith("/genres/")) return "Genre";
  const entry = NAV.find((candidate) => isActive(pathname, candidate));
  return entry?.label ?? "Music Player";
};
