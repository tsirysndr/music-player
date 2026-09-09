import gql from "graphql-tag";

import { ACCOUNT_FRAGMENT } from "./Query";

export const SIGN_IN = gql`
  mutation SignIn($handle: String!, $password: String!) {
    signIn(handle: $handle, password: $password) {
      ...AccountFragment
    }
  }
  ${ACCOUNT_FRAGMENT}
`;

export const SIGN_OUT = gql`
  mutation SignOut {
    signOut
  }
`;
