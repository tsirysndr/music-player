import _ from "lodash";
import { useQueryClient } from "@tanstack/react-query";
import { useAtomValue, useSetAtom } from "jotai";
import { resourceUriResolver } from "../ResourceUriResolver";
import { nowPlayingAtom, playbackIndexAtom, playbackPositionAtom } from "../State";
import { Track } from "../Types";
import {
  PlayAlbumMutationVariables,
  PlayArtistTracksMutationVariables,
  PlayNextMutationVariables,
  PlayPlaylistMutationVariables,
  PlayTrackAtMutationVariables,
  RemoveTrackAtMutationVariables,
  useCurrentlyPlayingSongQuery,
  useGetTracklistQuery,
  useNextMutation,
  usePauseMutation,
  usePlayAlbumMutation,
  usePlayArtistTracksMutation,
  usePlayMutation,
  usePlayNextMutation,
  usePlayPlaylistMutation,
  usePlayTrackAtMutation,
  usePreviousMutation,
  useRemoveTrackAtMutation,
  useSeekMutation,
} from "./GraphQL";

const toTrack = (track: any): Track => ({
  id: track.id,
  artist: track.artists.map((artist: any) => artist.name).join(", "),
  title: track.title,
  album: track.album.title,
  duration: track.duration!,
  artistId: _.get(track, "artists.0.id", ""),
  cover:
    track.album.cover && !_.startsWith(track.album.cover, "https://")
      ? resourceUriResolver.resolve(`/covers/${track.album.cover}`)
      : track.album.cover!,
});

export const usePlayback = () => {
  const nowPlaying = useAtomValue(nowPlayingAtom);
  const index = useAtomValue(playbackIndexAtom);
  const setNowPlaying = useSetAtom(nowPlayingAtom);
  const setPosition = useSetAtom(playbackPositionAtom);
  const queryClient = useQueryClient();

  const invalidatePlayback = () => {
    queryClient.invalidateQueries({
      queryKey: useCurrentlyPlayingSongQuery.getKey(),
    });
    queryClient.invalidateQueries({
      queryKey: useGetTracklistQuery.getKey(),
    });
  };

  const playMutation = usePlayMutation({ onSuccess: invalidatePlayback });
  const pauseMutation = usePauseMutation({ onSuccess: invalidatePlayback });
  const nextMutation = useNextMutation({ onSuccess: invalidatePlayback });
  const previousMutation = usePreviousMutation({
    onSuccess: invalidatePlayback,
  });
  const playTrackAtMutation = usePlayTrackAtMutation({
    onSuccess: invalidatePlayback,
  });
  const removeTrackAtMutation = useRemoveTrackAtMutation({
    onSuccess: invalidatePlayback,
  });
  const playNextMutation = usePlayNextMutation({
    onSuccess: invalidatePlayback,
  });
  const playAlbumMutation = usePlayAlbumMutation({
    onSuccess: invalidatePlayback,
  });
  const playArtistTracksMutation = usePlayArtistTracksMutation({
    onSuccess: invalidatePlayback,
  });
  const playPlaylistMutation = usePlayPlaylistMutation({
    onSuccess: invalidatePlayback,
  });
  const seekMutation = useSeekMutation();

  const { data: queue } = useGetTracklistQuery(undefined, {
    refetchInterval: 5000,
  });

  const nextTracks: Track[] =
    queue?.tracklistTracks?.nextTracks?.map(toTrack) || [];
  const previousTracks: Track[] =
    queue?.tracklistTracks?.previousTracks?.map(toTrack) || [];

  const play = () => {
    setNowPlaying((previous) => ({ ...previous, isPlaying: true }));
    return playMutation.mutateAsync({});
  };

  const pause = () => {
    setNowPlaying((previous) => ({ ...previous, isPlaying: false }));
    return pauseMutation.mutateAsync({});
  };

  const seek = (positionMs: number) => {
    // optimistic update of the position, the trackTimePosition
    // subscription will keep it in sync afterwards
    setPosition(positionMs);
    return seekMutation.mutateAsync({ position: positionMs });
  };

  return {
    nowPlaying,
    index,
    nextTracks,
    previousTracks,
    play,
    pause,
    seek,
    next: () => nextMutation.mutateAsync({}),
    previous: () => previousMutation.mutateAsync({}),
    playTrackAt: (variables: PlayTrackAtMutationVariables) =>
      playTrackAtMutation.mutateAsync(variables),
    removeTrackAt: (variables: RemoveTrackAtMutationVariables) =>
      removeTrackAtMutation.mutateAsync(variables),
    playNext: (variables: PlayNextMutationVariables) =>
      playNextMutation.mutateAsync(variables),
    playAlbum: (variables: PlayAlbumMutationVariables) =>
      playAlbumMutation.mutateAsync(variables),
    playArtistTracks: (variables: PlayArtistTracksMutationVariables) =>
      playArtistTracksMutation.mutateAsync(variables),
    playPlaylist: (variables: PlayPlaylistMutationVariables) =>
      playPlaylistMutation.mutateAsync(variables),
  };
};
