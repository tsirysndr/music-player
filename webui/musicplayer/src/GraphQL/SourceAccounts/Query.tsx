import gql from "graphql-tag";

// Revisions, offsets and large season/episode numbers stay decimal strings.
// Never round-trip these through Number or parseInt. The server validates them.
export type SourceDecimal = string;

export type ExtensionSourceDefinition = {
  extensionId: string;
  name: string;
  enabled: boolean;
  sourceApiVersion: number;
  // Values are write-only; these are declared keys, not guessed credentials.
  configKeys: string[];
};

export type ExtensionSourceAccount = {
  id: string;
  extensionId: string;
  name: string;
  sourceApiVersion: number;
  configRevision: SourceDecimal;
  // Binding is stored identity evidence, not a connectivity/health check.
  identityBound: boolean;
  overriddenConfigKeys: string[];
};

export type SourceBrowseList = "TRACKS" | "ALBUMS" | "ARTISTS" | "PLAYLISTS";

// V1 cannot recover this metadata by item ID. Retain it for full-track queue
// mutations. Album tags do not establish an album ID or a navigation link.
export type SourceMediaTrack = {
  id: string;
  title: string;
  artist: string;
  uri: string;
  duration: number | null;
  discNumber: number;
  trackNumber: number | null;
  album: { title: string; artist: string; year: number | null };
};

export type SourceMediaEntry = {
  id: string;
  title: string;
  itemType: string | null;
  mediaType: string | null;
  isContainer: boolean;
  seasonNumber: SourceDecimal | null;
  episodeNumber: SourceDecimal | null;
  track: SourceMediaTrack | null;
};

export type SourceMediaPage = {
  accountId: string;
  entries: SourceMediaEntry[];
  nextOffset: SourceDecimal | null;
};

export type ExtensionSourceDefinitionsQuery = {
  extensionSourceDefinitions: ExtensionSourceDefinition[];
};
export type SourceAccountsQuery = { sourceAccounts: ExtensionSourceAccount[] };
export type SourceAccountQuery = { sourceAccount: ExtensionSourceAccount };
export type SourceAccountVariables = { id: string };
export type BrowseSourceMediaQuery = { browseSourceMedia: SourceMediaPage };
export type BrowseSourceMediaVariables = {
  accountId: string;
  parent?: string | null;
  // Required for v1, omitted/null for v2. Never guess a list from a container.
  list?: SourceBrowseList | null;
  query?: string | null;
  // Omitted/null means "0"; use nextOffset verbatim for subsequent pages.
  offset?: SourceDecimal | null;
  // Server default 100; supplied values must be integers in 1..500.
  limit?: number | null;
};

export const SOURCE_ACCOUNT_FIELDS = gql`
  fragment SourceAccountFields on ExtensionSourceAccount {
    id
    extensionId
    name
    sourceApiVersion
    configRevision
    identityBound
    overriddenConfigKeys
  }
`;

export const GET_EXTENSION_SOURCE_DEFINITIONS = gql`
  query ExtensionSourceDefinitions {
    extensionSourceDefinitions {
      extensionId
      name
      enabled
      sourceApiVersion
      configKeys
    }
  }
`;

export const GET_SOURCE_ACCOUNTS = gql`
  query SourceAccounts {
    sourceAccounts {
      ...SourceAccountFields
    }
  }
  ${SOURCE_ACCOUNT_FIELDS}
`;

export const GET_SOURCE_ACCOUNT = gql`
  query SourceAccount($id: ID!) {
    sourceAccount(id: $id) {
      ...SourceAccountFields
    }
  }
  ${SOURCE_ACCOUNT_FIELDS}
`;

export const BROWSE_SOURCE_MEDIA = gql`
  query BrowseSourceMedia(
    $accountId: ID!
    $parent: ID
    $list: SourceBrowseList
    $query: String
    $offset: String
    $limit: Int
  ) {
    browseSourceMedia(
      accountId: $accountId
      parent: $parent
      list: $list
      query: $query
      offset: $offset
      limit: $limit
    ) {
      accountId
      nextOffset
      entries {
        id
        title
        itemType
        mediaType
        isContainer
        seasonNumber
        episodeNumber
        track {
          id
          title
          artist
          uri
          duration
          discNumber
          trackNumber
          album {
            title
            artist
            year
          }
        }
      }
    }
  }
`;
