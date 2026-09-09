export type Track = {
  id: string;
  title: string;
  artist: string;
  album: string;
  /**
   * Seconds, as the daemon stores it — fractional, not rounded.
   *
   * Note this is *not* the unit `NowPlaying.duration` uses: that one is
   * milliseconds, because it is compared against a playback position. Anything
   * handing this to `formatTime` has to multiply by 1000 first.
   */
  duration: number;
  cover?: string;
  artistId: string;
  albumId?: string;
  /**
   * Musical key in Camelot notation, e.g. "8A", and the tempo.
   *
   * Null unless the daemon's own library has analysed the track — a remote
   * Subsonic or Jellyfin server has no such field to give, which is why the
   * columns are hidden entirely for those rather than shown empty.
   */
  key?: string | null;
  bpm?: number | null;
};
