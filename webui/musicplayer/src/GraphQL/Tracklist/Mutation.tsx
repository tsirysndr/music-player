import { gql } from "@apollo/client";

export const CLEAR_TRACKLIST = gql`
  mutation ClearTracklist {
    clearTracklist
  }
`;

export const PLAY_ALBUM = gql`
  mutation PlayAlbum($albumId: ID!, $position: Int, $shuffle: Boolean!) {
    playAlbum(id: $albumId, position: $position, shuffle: $shuffle)
  }
`;

export const PLAY_ARTIST_TRACKS = gql`
  mutation PlayArtistTracks(
    $artistId: ID!
    $position: Int
    $shuffle: Boolean!
  ) {
    playArtistTracks(id: $artistId, position: $position, shuffle: $shuffle)
  }
`;

export const PLAY_PLAYLIST = gql`
  mutation PlayPlaylist($playlistId: ID!, $position: Int, $shuffle: Boolean!) {
    playPlaylist(id: $playlistId, position: $position, shuffle: $shuffle)
  }
`;

export const PLAY_TRACK_AT = gql`
  mutation PlayTrackAt($position: Int!) {
    playTrackAt(position: $position)
  }
`;

export const REMOVE_TRACK_AT = gql`
  mutation RemoveTrackAt($position: Int!) {
    removeTrack(position: $position)
  }
`;

export const PLAY_NEXT = gql`
  mutation PlayNext($trackId: ID!) {
    playNext(id: $trackId)
  }
`;
