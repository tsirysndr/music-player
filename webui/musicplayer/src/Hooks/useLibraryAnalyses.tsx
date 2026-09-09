import { useGetConnectedServerQuery } from "./GraphQL";

/**
 * Whether the connected library can report a key and a tempo.
 *
 * Only a music-player daemon analyses its own tracks — a Subsonic, Jellyfin,
 * Plex or Kodi server has no such field to give. The columns are hidden for
 * those rather than shown blank on every row, which would read as a broken
 * feature instead of an absent one.
 *
 * `null` for the connected server means the daemon's own library, which of
 * course analyses.
 */
export const useLibraryAnalyses = (): boolean => {
  const { data, isPending } = useGetConnectedServerQuery();
  // Assumed false until the answer arrives: a column that appears is far less
  // jarring than two that appear and then vanish.
  if (isPending) return false;
  const server = data?.connectedServer;
  return !server || server.kind === "music-player";
};
