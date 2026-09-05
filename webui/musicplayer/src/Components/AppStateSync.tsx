import { FC } from "react";
import { useDevicesSync } from "../Hooks/useDevicesSync";
import { usePlaybackSync } from "../Hooks/usePlaybackSync";

/**
 * Mounted once inside the router; keeps the jotai atoms (playback,
 * devices) in sync with the server via react-query + GraphQL
 * subscriptions.
 */
const AppStateSync: FC = () => {
  usePlaybackSync();
  useDevicesSync();
  return null;
};

export default AppStateSync;
