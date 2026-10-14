import { useCallback } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { usePlayTrackMutation } from "./GraphQL";

/**
 * Play one library track now.
 *
 * The daemon owns queue replacement and playback startup as one operation.
 * Keeping that transition atomic avoids acknowledging an intermediate
 * `playNext` command before the requested track has actually been selected.
 */
export const usePlayTrack = () => {
  const queryClient = useQueryClient();
  const { mutateAsync } = usePlayTrackMutation({
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["GetTracklist"] });
      void queryClient.invalidateQueries({ queryKey: ["GetCurrentTrack"] });
    },
  });

  return useCallback(
    (trackId: string) => mutateAsync({ trackId }),
    [mutateAsync]
  );
};
