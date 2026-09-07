import gql from "graphql-tag";

/**
 * Switch an extension on or off.
 *
 * The whole row comes back so the page renders what was stored rather than
 * what was asked for — the two differ if the daemon refuses.
 */
export const SET_EXTENSION_ENABLED = gql`
  mutation SetExtensionEnabled($id: ID!, $enabled: Boolean!) {
    setExtensionEnabled(id: $id, enabled: $enabled) {
      id
      name
      version
      author
      description
      homepage
      repository
      license
      logo
      topics
      capabilities
      allowedHosts
      libraryRead
      status
      path
    }
  }
`;

/**
 * Re-read the extension directories, for an extension added or removed on disk
 * while the page is open. Also drops the stored flags for anything gone.
 */
export const RESCAN_EXTENSIONS = gql`
  mutation RescanExtensions($filter: String) {
    rescanExtensions(filter: $filter) {
      id
      name
      version
      author
      description
      homepage
      repository
      license
      logo
      topics
      capabilities
      allowedHosts
      libraryRead
      status
      path
    }
  }
`;
