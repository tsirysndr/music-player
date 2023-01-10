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
