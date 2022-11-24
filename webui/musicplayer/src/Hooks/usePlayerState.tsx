import { useEffect } from "react";
import { useGetPlayerStateQuery } from "./GraphQL";

export const usePlayerState = () => {
  const {
    data: playerState,
    startPolling,
    stopPolling,
  } = useGetPlayerStateQuery({
    pollInterval: 500,
  });
  useEffect(() => {
    startPolling(500);
    return () => stopPolling();
  });
  return {
    playerState: playerState?.getPlayerState,
  };
};
