import { gql } from "@apollo/client";

export const CONNECT_TO_DEVICE = gql`
  mutation ConnectToDevice($id: ID!) {
    connectToDevice(id: $id) {
      id
    }
  }
`;

export const DISCONNECT_FROM_DEVICE = gql`
  mutation DisconnectFromDevice {
    disconnectFromDevice {
      id
    }
  }
`;

export const CONNECT_TO_CAST_DEVICE = gql`
  mutation ConnectToCastDevice($id: ID!) {
    connectToCastDevice(id: $id) {
      id
    }
  }
`;

export const DISCONNECT_FROM_CAST_DEVICE = gql`
  mutation DisconnectFromCastDevice {
    disconnectFromCastDevice {
      id
    }
  }
`;
