/**
 * Sample data shared by the stories.
 *
 * One place for it so a change to a component's props shows up as one
 * compile error rather than six, and so every story shows the same library.
 */
import type {
  AlbumCardItem,
  ArtistRowItem,
  PlaylistOption,
  PlaylistRowItem,
  TrackRowItem,
} from "../Components/UI";

const TIDAL = "https://resources.tidal.com/images";

export const albums: AlbumCardItem[] = [
  {
    id: "1",
    title: "Californication",
    artist: "Red Hot Chili Peppers",
    year: 1999,
    cover: `${TIDAL}/9be9e74c/1b06/479a/a79e/5c62733d74ac/320x320.jpg`,
  },
  {
    id: "2",
    title: "Discovery",
    artist: "Daft Punk",
    year: 2001,
    cover: `${TIDAL}/f87d9afc/075e/43f4/bbbc/7770b46cb8aa/320x320.jpg`,
  },
  {
    id: "3",
    title: "Iowa",
    artist: "Slipknot",
    year: 2001,
    cover: `${TIDAL}/c3672adf/ca16/4db3/a375/64904d1f7d39/320x320.jpg`,
  },
  // No cover, so the skin's art placeholder and disc glyph are exercised too.
  { id: "4", title: "Definitely Maybe", artist: "Oasis", year: 1994 },
];

export const artists: ArtistRowItem[] = [
  {
    id: "1",
    name: "Red Hot Chili Peppers",
    picture: `${TIDAL}/9be9e74c/1b06/479a/a79e/5c62733d74ac/320x320.jpg`,
  },
  {
    id: "2",
    name: "Daft Punk",
    picture: `${TIDAL}/f87d9afc/075e/43f4/bbbc/7770b46cb8aa/320x320.jpg`,
  },
  { id: "3", name: "Oasis" },
];

export const tracks: TrackRowItem[] = [
  {
    id: "t1",
    trackNumber: 1,
    title: "Around the World",
    artist: "Daft Punk",
    artistId: "2",
    album: "Homework",
    albumId: "5",
    duration: "07:09",
    liked: true,
  },
  {
    id: "t2",
    trackNumber: 2,
    title: "Otherside",
    artist: "Red Hot Chili Peppers",
    artistId: "1",
    album: "Californication",
    albumId: "1",
    duration: "04:15",
  },
  {
    id: "t3",
    trackNumber: 3,
    title: "Road Trippin'",
    artist: "Red Hot Chili Peppers",
    artistId: "1",
    album: "Californication",
    albumId: "1",
    duration: "03:25",
  },
  {
    id: "t4",
    trackNumber: 4,
    title:
      "A title long enough that the column has to elide it rather than push the others out of line",
    artist: "Some Artist With A Long Name",
    artistId: "3",
    album: "An Album With A Long Name",
    albumId: "6",
    duration: "12:48",
  },
];

export const playlists: PlaylistRowItem[] = [
  {
    id: "p1",
    name: "Late night",
    description: "For the small hours",
    trackCount: 42,
  },
  { id: "p2", name: "Focus", trackCount: 1 },
  { id: "p3", name: "Rediscover", description: "Smart · year<2005", trackCount: 120 },
];

export const recentPlaylists: PlaylistOption[] = playlists.map((playlist) => ({
  id: playlist.id,
  name: playlist.name,
}));

export const noop = () => {};
