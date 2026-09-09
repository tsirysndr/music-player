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
    picture
  }
`;

export const TRACK_FRAGMENT = gql`
  fragment TrackFragment on Track {
    id
    trackNumber
    title
    artist
    duration
    # Null unless the daemon's own library has analysed the track. Selected in
    # the fragment so every list that shows tracks can show the columns, rather
    # than only the ones that remembered to ask.
    key
    bpm
  }
`;

export const PLAYLIST_FRAGMENT = gql`
  fragment PlaylistFragment on Playlist {
    id
    name
    description
    tracks {
      id
      title
      albumTitle
      artist
      artistId
      albumId
      cover
      duration
      key
      bpm
    }
  }
`;

export const FOLDER_FRAGMENT = gql`
  fragment FolderFragment on Folder {
    id
    name
    playlists {
      id
      name
      description
    }
  }
`;
