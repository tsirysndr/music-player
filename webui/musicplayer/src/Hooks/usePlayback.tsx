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
  usePlayPlaylistMutation,
  useCurrentlyPlayingSongChangedSubscription,
  usePlayerStateChangedSubscription,
  useGetPlayerStateQuery,
} from "./GraphQL";

export const usePlayback = () => {
  const {
    data: playback,
    startPolling: startPollingCurrentSong,
    stopPolling: stopPollingCurrentSong,
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
  const {
    data: playerStateQueryData,
    startPolling: startPollingPlayerState,
    stopPolling: stopPollingPlayerState,
  } = useGetPlayerStateQuery({
    pollInterval: 500,
  });
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
  const isPlaying = playerStateQueryData?.getPlayerState?.isPlaying;
  const currentTrack =
    currentlyPlayingSongData?.currentlyPlayingSong ||
    playback?.currentlyPlayingSong;
  const duration = playback?.currentlyPlayingSong?.duration! * 1000;
  let nowPlaying = {
    id: currentTrack?.id,
    title: currentTrack?.title,
    artist:
      currentTrack?.artist ||
      currentTrack?.artists?.map((artist) => artist.name).join(", "),
    album: currentTrack?.album?.title,
    duration,
    cover: `/covers/${currentTrack?.album?.id}.jpg`,
    albumId: currentTrack?.album?.id,
    isPlaying,
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
    startPollingCurrentSong!(1000);
    startPollingTracklist(1000);
    startPollingPlayerState(1000);
    return () => {
      stopPollingCurrentSong();
      stopPollingTracklist();
      stopPollingPlayerState();
    };
  });

  return {
    nowPlaying,
    play,
    pause,
    next,
    previous,
    nextTracks,
    previousTracks,
    playTrackAt,
    removeTrackAt,
    playNext,
    playAlbum,
    playArtistTracks,
    playPlaylist,
    currentTrackId: nowPlaying.id,
    isPlaying,
  };
};
