import gql from "graphql-tag";
import { ALBUM_FRAGMENT, ARTIST_FRAGMENT, TRACK_FRAGMENT } from "../Fragments";

export const GET_ALBUMS = gql`
  query GetAlbums($filter: String, $offset: Int, $limit: Int) {
    albums(filter: $filter, offset: $offset, limit: $limit) {
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
  query GetArtists($filter: String, $offset: Int, $limit: Int) {
    artists(filter: $filter, offset: $offset, limit: $limit) {
      ...ArtistFragment
    }
  }
  ${ARTIST_FRAGMENT}
`;

export const GET_TRACKS = gql`
  query GetTracks($filter: String, $offset: Int, $limit: Int) {
    tracks(filter: $filter, offset: $offset, limit: $limit) {
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
        discNumber
        title
        artist
        artists {
          id
          name
        }
        duration
        uri
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
        source
      }
      albums {
        id
        title
        artist
        cover
        source
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
        source
      }
    }
  }
`;
