import { useEffect } from "react";
import { useGetPlaybackProgressQuery } from "./GraphQL";

export const usePlayackProgress = () => {
  const { data, startPolling, stopPolling } = useGetPlaybackProgressQuery({
    pollInterval: 500,
  });

  useEffect(() => {
    startPolling(500);
    return () => stopPolling();
  });

  return { progress: data?.playbackProgress };
};
