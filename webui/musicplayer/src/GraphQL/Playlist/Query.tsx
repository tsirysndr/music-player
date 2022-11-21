import { gql } from "@apollo/client";
import { FOLDER_FRAGMENT, PLAYLIST_FRAGMENT } from "../Fragments";

export const GET_PLAYLISTS = gql`
  query GetPlaylists {
    playlists {
      id
      name
      description
    }
  }
`;

export const GET_PLAYLIST = gql`
  query GetPlaylist($id: ID!) {
    playlist(id: $id) {
      ...PlaylistFragment
    }
  }
  ${PLAYLIST_FRAGMENT}
`;

export const GET_FOLDERS = gql`
  query GetFolders {
    folders {
      id
      name
    }
  }
  ${FOLDER_FRAGMENT}
`;

export const GET_FOLDER = gql`
  query GetFolder($id: ID!) {
    folder(id: $id) {
      ...FolderFragment
    }
  }
  ${FOLDER_FRAGMENT}
`;
