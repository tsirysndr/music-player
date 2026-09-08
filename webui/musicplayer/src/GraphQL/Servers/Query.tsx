import gql from "graphql-tag";

export const SERVER_FRAGMENT = gql`
  fragment ServerFragment on Server {
    id
    kind
    name
    url
    username
    hasPassword
    connected
  }
`;

export const GET_SAVED_SERVERS = gql`
  query GetSavedServers {
    savedServers {
      ...ServerFragment
    }
  }
  ${SERVER_FRAGMENT}
`;

/**
 * The kinds of server this daemon can talk to, straight from its provider
 * registry — so a backend added on the Rust side turns up in the add-server
 * form without this client being changed.
 */
export const GET_SOURCE_KINDS = gql`
  query GetSourceKinds {
    sourceKinds {
      kind
      displayName
      needsCredentials
      defaultPort
      fixedUrl
    }
  }
`;

/** `null` means the library screens are reading local files. */
export const GET_CONNECTED_SERVER = gql`
  query GetConnectedServer {
    connectedServer {
      ...ServerFragment
    }
  }
  ${SERVER_FRAGMENT}
`;
