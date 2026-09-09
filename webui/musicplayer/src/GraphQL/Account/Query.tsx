import gql from "graphql-tag";

export const ACCOUNT_FRAGMENT = gql`
  fragment AccountFragment on Account {
    did
    handle
    displayName
    avatar
  }
`;

/** `null` means nobody is signed in. */
export const GET_ACCOUNT = gql`
  query GetAccount {
    account {
      ...AccountFragment
    }
  }
  ${ACCOUNT_FRAGMENT}
`;
