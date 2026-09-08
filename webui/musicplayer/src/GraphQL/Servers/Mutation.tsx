import gql from "graphql-tag";

import { SERVER_FRAGMENT } from "./Query";

export const ADD_SERVER = gql`
  mutation AddServer($input: ServerInput!) {
    addServer(input: $input) {
      ...ServerFragment
    }
  }
  ${SERVER_FRAGMENT}
`;

export const DELETE_SERVER = gql`
  mutation DeleteServer($id: ID!) {
    deleteServer(id: $id)
  }
`;

export const CONNECT_TO_SERVER = gql`
  mutation ConnectToServer($id: ID!) {
    connectToServer(id: $id) {
      ...ServerFragment
    }
  }
  ${SERVER_FRAGMENT}
`;

export const DISCONNECT_FROM_SERVER = gql`
  mutation DisconnectFromServer {
    disconnectFromServer {
      id
      name
    }
  }
`;
