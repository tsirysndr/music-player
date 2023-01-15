import { gql } from "@apollo/client";

export const LIST_DEVICES = gql`
  query ListDevices {
    listDevices {
      id
      app
      name
      service
      host
      port
      isConnected
    }
  }
`;

export const LIST_CAST_DEVICES = gql`
  query ListCastDevices {
    listCastDevices {
      id
      app
      name
      service
      host
      port
      isConnected
    }
  }
`;

export const CONNECTED_DEVICE = gql`
  query ConnectedDevice {
    connectedDevice {
      id
      name
      app
      host
      port
      isConnected
    }
  }
`;
