import gql from "graphql-tag";

export const ALBUM_FRAGMENT = gql`
  fragment AlbumFragment on Album {
    id
    title
    artist
    year
    cover
  }
`;

export const ARTIST_FRAGMENT = gql`
  fragment ArtistFragment on Artist {
    id
    name
  }
`;

export const TRACK_FRAGMENT = gql`
  fragment TrackFragment on Track {
    id
    trackNumber
    title
    duration
  }
`;
