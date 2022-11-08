import _ from "lodash";
import { useEffect } from "react";
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
  const {
    data: queue,
    startPolling: startPollingTracklist,
    stopPolling: stopPollingTracklist,
  } = useGetTracklistQuery({
    pollInterval: 500,
  });
  const index = playback?.currentlyPlayingSong.index;
  const duration = playback?.currentlyPlayingSong?.track?.duration! * 1000;
  const position = playback?.currentlyPlayingSong?.positionMs!;
  const nowPlaying = {
    id: playback?.currentlyPlayingSong?.track?.id,
    title: playback?.currentlyPlayingSong?.track?.title,
    artist: playback?.currentlyPlayingSong?.track?.artists
      ?.map((artist) => artist.name)
      .join(", "),
    album: playback?.currentlyPlayingSong?.track?.album?.title,
    isPlaying: playback?.currentlyPlayingSong?.isPlaying,
    duration,
    progress: position,
    cover: `/covers/${playback?.currentlyPlayingSong?.track?.album?.id}.jpg`,
    albumId: playback?.currentlyPlayingSong?.track?.album?.id,
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
          ? `/covers/${track.album.cover}`
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
          ? `/covers/${track.album.cover}`
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
  };
};
