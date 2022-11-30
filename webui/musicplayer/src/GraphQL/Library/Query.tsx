import gql from "graphql-tag";
import { ALBUM_FRAGMENT, ARTIST_FRAGMENT, TRACK_FRAGMENT } from "../Fragments";

export const GET_ALBUMS = gql`
  query GetAlbums($limit: Int, $offset: Int) {
    albums(limit: $limit, offset: $offset) {
      ...AlbumFragment
      tracks {
        ...TrackFragment
      }
    }
  }
  ${ALBUM_FRAGMENT}
  ${TRACK_FRAGMENT}
`;

export const GET_ARTISTS = gql`
  query GetArtists($limit: Int, $offset: Int) {
    artists(limit: $limit, offset: $offset) {
      ...ArtistFragment
    }
  }
  ${ARTIST_FRAGMENT}
`;

export const GET_TRACKS = gql`
  query GetTracks($limit: Int, $offset: Int) {
    tracks(limit: $limit, offset: $offset) {
      ...TrackFragment
      artists {
        id
        name
      }
      album {
        ...AlbumFragment
      }
    }
  }
  ${TRACK_FRAGMENT}
  ${ARTIST_FRAGMENT}
  ${ALBUM_FRAGMENT}
`;

export const GET_ARTIST = gql`
  query GetArtist($id: ID!) {
    artist(id: $id) {
      id
      name
      picture
      songs {
        id
        title
        artist
        artists {
          id
          name
        }
        album {
          id
          title
          cover
        }
        duration
      }
      albums {
        ...AlbumFragment
      }
    }
  }
  ${ALBUM_FRAGMENT}
`;

export const GET_ALBUM = gql`
  query GetAlbum($id: ID!) {
    album(id: $id) {
      ...AlbumFragment
      tracks {
        id
        trackNumber
        title
        artist
        artists {
          id
          name
        }
        duration
      }
    }
  }
  ${ALBUM_FRAGMENT}
`;

export const SEARCH = gql`
  query Search($keyword: String!) {
    search(keyword: $keyword) {
      artists {
        id
        name
        picture
      }
      albums {
        id
        title
        artist
        cover
      }
      tracks {
        id
        title
        artist
        duration
        cover
        artistId
        albumId
        albumTitle
      }
    }
  }
`;
