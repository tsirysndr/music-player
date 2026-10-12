import gql from "graphql-tag";
import {
  SOURCE_ACCOUNT_FIELDS,
  type ExtensionSourceAccount,
  type SourceDecimal,
} from "./Query";

export type SourceConfigValueInput = { key: string; value: string };

// Keep is represented by omitting the key from changes. SET must include its
// value, including "". REMOVE omits value and restores the extension default.
export type SourceConfigChangeInput =
  | { key: string; operation: "SET"; value: string }
  | { key: string; operation: "REMOVE"; value?: never };

export type CreateSourceAccountVariables = {
  extensionId: string;
  name: string;
  // Omitted keys use declared extension defaults; empty values are overrides.
  config?: SourceConfigValueInput[] | null;
};
export type CreateSourceAccountMutation = {
  createSourceAccount: ExtensionSourceAccount;
};

export type UpdateSourceAccountVariables = {
  id: string;
  expectedRevision: SourceDecimal;
  name?: string | null;
  changes?: SourceConfigChangeInput[] | null;
};
export type UpdateSourceAccountMutation = {
  updateSourceAccount: ExtensionSourceAccount;
};

export type ConnectSourceAccountVariables = { id: string };
export type ConnectSourceAccountMutation = {
  connectSourceAccount: ExtensionSourceAccount;
};
export type DeleteSourceAccountVariables = { id: string };
export type DeleteSourceAccountMutation = { deleteSourceAccount: boolean };

// Saving and connecting are separate explicit operations. A lost response can
// still mean a committed mutation; callers must not automatically retry these.
export const CREATE_SOURCE_ACCOUNT = gql`
  mutation CreateSourceAccount(
    $extensionId: ID!
    $name: String!
    $config: [SourceConfigValueInput!]
  ) {
    createSourceAccount(extensionId: $extensionId, name: $name, config: $config) {
      ...SourceAccountFields
    }
  }
  ${SOURCE_ACCOUNT_FIELDS}
`;

export const UPDATE_SOURCE_ACCOUNT = gql`
  mutation UpdateSourceAccount(
    $id: ID!
    $expectedRevision: String!
    $name: String
    $changes: [SourceConfigChangeInput!]
  ) {
    updateSourceAccount(
      id: $id
      expectedRevision: $expectedRevision
      name: $name
      changes: $changes
    ) {
      ...SourceAccountFields
    }
  }
  ${SOURCE_ACCOUNT_FIELDS}
`;

export const CONNECT_SOURCE_ACCOUNT = gql`
  mutation ConnectSourceAccount($id: ID!) {
    connectSourceAccount(id: $id) {
      ...SourceAccountFields
    }
  }
  ${SOURCE_ACCOUNT_FIELDS}
`;

export const DELETE_SOURCE_ACCOUNT = gql`
  mutation DeleteSourceAccount($id: ID!) {
    deleteSourceAccount(id: $id)
  }
`;
