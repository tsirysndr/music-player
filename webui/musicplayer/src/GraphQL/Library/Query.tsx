import gql from "graphql-tag";
import { ALBUM_FRAGMENT, ARTIST_FRAGMENT, TRACK_FRAGMENT } from "../Fragments";

export const GET_ALBUMS = gql`
  query GetAlbums {
    albums {
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
  query GetArtists {
    artists {
      ...ArtistFragment
    }
  }
  ${ARTIST_FRAGMENT}
`;

export const GET_TRACKS = gql`
  query GetTracks {
    tracks {
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
    }
  }
`;

export const GET_ALBUM = gql`
  query GetAlbum($id: ID!) {
    album(id: $id) {
      ...AlbumFragment
      tracks {
        id
        trackNumber
        title
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
