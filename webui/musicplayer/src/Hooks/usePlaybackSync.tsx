import { useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { useSetAtom } from "jotai";
import { resourceUriResolver } from "../ResourceUriResolver";
import {
  nowPlayingAtom,
  playbackIndexAtom,
  serverConnectedAtom,
} from "../State";
import {
  CurrentlyPlayingSongChangedDocument,
  CurrentlyPlayingSongChangedSubscription,
  PlayerStateChangedDocument,
  PlayerStateChangedSubscription,
  TrackTimePositionChangedDocument,
  TrackTimePositionChangedSubscription,
  useCurrentlyPlayingSongQuery,
  useGetTracklistQuery,
} from "./GraphQL";
import { useGraphQLSubscription } from "./useGraphQLSubscription";

/**
 * Keeps the playback atoms in sync with the server. Should be mounted
 * once (see AppStateSync). The currently playing song is seeded with a
 * react-query query (modest 5s refetch interval as a fallback) and kept
 * live via the trackTimePosition / playerState / currentlyPlayingSong
 * GraphQL subscriptions.
 */
export const usePlaybackSync = () => {
  const setNowPlaying = useSetAtom(nowPlayingAtom);
  const setPlaybackIndex = useSetAtom(playbackIndexAtom);
  const queryClient = useQueryClient();

  const setConnected = useSetAtom(serverConnectedAtom);

  const { data: playback, isError } = useCurrentlyPlayingSongQuery(undefined, {
    refetchInterval: 5000,
  });

  // The 5s poll doubles as the heartbeat behind the sidebar's status dot.
  useEffect(() => {
    setConnected(!isError);
  }, [isError, setConnected]);

  useEffect(() => {
    if (!playback?.currentlyPlayingSong) {
      return;
    }
    const { track, index, isPlaying, positionMs } =
      playback.currentlyPlayingSong;
    setPlaybackIndex(index);
    setNowPlaying({
      id: track?.id,
      title: track?.title,
      artist:
        track?.artist || track?.artists?.map((artist) => artist.name).join(", "),
      album: track?.album?.title,
      albumId: track?.album?.id,
      cover: resourceUriResolver.resolve(`/covers/${track?.album?.cover}`),
      duration: (track?.duration || 0) * 1000,
      progress: positionMs,
      isPlaying,
    });
  }, [playback, setNowPlaying, setPlaybackIndex]);

  useGraphQLSubscription<TrackTimePositionChangedSubscription>(
    TrackTimePositionChangedDocument,
    undefined,
    (data) => {
      if (!data.trackTimePosition) return;
      const { positionMs } = data.trackTimePosition;
      setNowPlaying((previous) => ({ ...previous, progress: positionMs }));
    }
  );

  useGraphQLSubscription<PlayerStateChangedSubscription>(
    PlayerStateChangedDocument,
    undefined,
    (data) => {
      if (!data.playerState) return;
      const { isPlaying } = data.playerState;
      setNowPlaying((previous) => ({ ...previous, isPlaying }));
    }
  );

  useGraphQLSubscription<CurrentlyPlayingSongChangedSubscription>(
    CurrentlyPlayingSongChangedDocument,
    undefined,
    (data) => {
      const track = data.currentlyPlayingSong;
      if (!track) return;
      setNowPlaying((previous) => ({
        ...previous,
        id: track.id,
        title: track.title,
        artist:
          track.artist || track.artists?.map((artist) => artist.name).join(", "),
        album: track.album?.title,
        albumId: track.album?.id,
        cover: resourceUriResolver.resolve(`/covers/${track.album?.cover}`),
        duration: (track.duration || 0) * 1000,
        progress: 0,
      }));
      queryClient.invalidateQueries({
        queryKey: useCurrentlyPlayingSongQuery.getKey(),
      });
      queryClient.invalidateQueries({
        queryKey: useGetTracklistQuery.getKey(),
      });
    }
  );
};
