import _ from "lodash";
import { useEffect } from "react";
import { resourceUriResolver } from "../ResourceUriResolver";
import { Track } from "../Types";
import {
  useCurrentlyPlayingSongQuery,
  usePlayMutation,
  usePauseMutation,
  useNextMutation,
  usePreviousMutation,
  useGetTracklistQuery,
  usePlayTrackAtMutation,
  useRemoveTrackAtMutation,
  usePlayNextMutation,
  usePlayAlbumMutation,
  usePlayArtistTracksMutation,
  usePlayPlaylistMutation,
  useTrackTimePositionChangedSubscription,
  useCurrentlyPlayingSongChangedSubscription,
  usePlayerStateChangedSubscription,
} from "./GraphQL";

export const usePlayback = () => {
  const {
    data: playback,
    startPolling,
    stopPolling,
  } = useCurrentlyPlayingSongQuery({
    pollInterval: 500,
  });
  const [play] = usePlayMutation();
  const [pause] = usePauseMutation();
  const [next] = useNextMutation();
  const [previous] = usePreviousMutation();
  const [playTrackAt] = usePlayTrackAtMutation();
  const [removeTrackAt] = useRemoveTrackAtMutation();
  const [playNext] = usePlayNextMutation();
  const [playAlbum] = usePlayAlbumMutation();
  const [playArtistTracks] = usePlayArtistTracksMutation();
  const [playPlaylist] = usePlayPlaylistMutation();
  const { data: currentlyPlayingSongData } =
    useCurrentlyPlayingSongChangedSubscription();
  const { data: playerStateData } = usePlayerStateChangedSubscription();
  const {
    data: queue,
    startPolling: startPollingTracklist,
    stopPolling: stopPollingTracklist,
  } = useGetTracklistQuery({
    pollInterval: 500,
  });
  const currentTrack =
    currentlyPlayingSongData?.currentlyPlayingSong ||
    playback?.currentlyPlayingSong?.track;
  const index = playback?.currentlyPlayingSong.index;
  const duration = playback?.currentlyPlayingSong?.track?.duration! * 1000;
  const position = playback?.currentlyPlayingSong?.positionMs!;
  let nowPlaying = {
    id: currentTrack?.id,
    title: currentTrack?.title,
    artist:
      currentTrack?.artist ||
      currentTrack?.artists?.map((artist) => artist.name).join(", "),
    album: currentTrack?.album?.title,
    duration,
    progress: position,
    cover: resourceUriResolver.resolve(`/covers/${currentTrack?.album?.id}.jpg`),
    albumId: currentTrack?.album?.id,
    isPlaying:
      playerStateData?.playerState?.isPlaying ||
      playback?.currentlyPlayingSong?.isPlaying,
  };
  const nextTracks: Track[] =
    queue?.tracklistTracks?.nextTracks?.map((track) => ({
      id: track.id,
      artist: track.artists.map((artist) => artist.name).join(", "),
      title: track.title,
      album: track.album.title,
      duration: track.duration!,
      artistId: _.get(track, "artists.0.id", ""),
      cover:
        track.album.cover && !_.startsWith(track.album.cover, "https://")
          ? resourceUriResolver.resolve(`/covers/${track.album.cover}`)
          : track.album.cover!,
    })) || [];
  const previousTracks: Track[] =
    queue?.tracklistTracks?.previousTracks?.map((track) => ({
      id: track.id,
      artist: track.artists.map((artist) => artist.name).join(", "),
      title: track.title,
      album: track.album.title,
      duration: track.duration!,
      artistId: _.get(track, "artists.0.id", ""),
      cover:
        track.album.cover && !_.startsWith(track.album.cover, "https://")
          ? resourceUriResolver.resolve(`/covers/${track.album.cover}`)
          : track.album.cover!,
    })) || [];
  useEffect(() => {
    startPolling!(1000);
    startPollingTracklist(1000);
    return () => {
      stopPolling();
      stopPollingTracklist();
    };
  }, [startPolling, stopPolling, startPollingTracklist, stopPollingTracklist]);
  return {
    nowPlaying,
    play,
    pause,
    next,
    previous,
    index,
    nextTracks,
    previousTracks,
    playTrackAt,
    removeTrackAt,
    playNext,
    playAlbum,
    playArtistTracks,
    playPlaylist,
  };
};
