export type Track = {
  trackNumber?: string;
  id: string;
  title: string;
  artist: string;
  album: string;
  duration: number;
  cover?: string;
  artistId: string;
  albumId?: string;
  liked?: boolean;
};
