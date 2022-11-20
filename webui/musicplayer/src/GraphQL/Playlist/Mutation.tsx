import { gql } from "@apollo/client";
import { FOLDER_FRAGMENT, PLAYLIST_FRAGMENT } from "../Fragments";

export const CREATE_PLAYLIST = gql`
  mutation CreatePlaylist($name: String!, $description: String) {
    createPlaylist(name: $name, description: $description) {
      id
      name
      description
    }
  }
`;

export const DELETE_PLAYLIST = gql`
  mutation DeletePlaylist($id: ID!) {
    deletePlaylist(id: $id) {
      id
    }
  }
`;

export const ADD_TRACK_TO_PLAYLIST = gql`
  mutation AddTrackToPlaylist($playlistId: ID!, $trackId: ID!) {
    addTrackToPlaylist(id: $playlistId, trackId: $trackId) {
      ...PlaylistFragment
    }
  }
  ${PLAYLIST_FRAGMENT}
`;

export const REMOVE_TRACK_FROM_PLAYLIST = gql`
  mutation RemoveTrackFromPlaylist($playlistId: ID!, $position: Int!) {
    removeTrackFromPlaylist(id: $playlistId, position: $position) {
      ...PlaylistFragment
    }
  }
`;

export const RENAME_PLAYLIST = gql`
  mutation RenamePlaylist($id: ID!, $name: String!) {
    renamePlaylist(id: $id, name: $name) {
      id
      name
    }
  }
`;

export const CREATE_FOLDER = gql`
  mutation CreateFolder($name: String!) {
    createFolder(name: $name) {
      id
      name
    }
  }
`;

export const RENAME_FOLDER = gql`
  mutation RenameFolder($id: ID!, $name: String!) {
    renameFolder(id: $id, name: $name) {
      id
      name
    }
  }
`;

export const DELETE_FOLDER = gql`
  mutation DeleteFolder($id: ID!) {
    deleteFolder(id: $id) {
      id
    }
  }
`;

export const MOVE_PLAYLIST_TO_FOLDER = gql`
  mutation MovePlaylistToFolder($playlistId: ID!, $folderId: ID!) {
    movePlaylistToFolder(id: $playlistId, folderId: $folderId) {
      ...FolderFragment
    }
  }
  ${FOLDER_FRAGMENT}
`;

export const MOVE_PLAYLISTS_TO_FOLDER = gql`
  mutation MovePlaylistsToFolder($playlistIds: [ID!]!, $folderId: ID!) {
    movePlaylistsToFolder(ids: $playlistIds, folderId: $folderId) {
      ...FolderFragment
    }
  }
  ${FOLDER_FRAGMENT}
`;
