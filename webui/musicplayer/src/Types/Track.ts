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
};
