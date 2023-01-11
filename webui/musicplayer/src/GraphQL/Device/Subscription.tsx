import { gql } from "@apollo/client";

export const NEW_DEVICE = gql`
  subscription OnNewDevice {
    onNewDevice {
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

export const ON_DEVICE_CONNECTED = gql`
  subscription OnDeviceConnected {
    onConnected {
      id
      name
    }
  }
`;

export const ON_DEVICE_DISCONNECTED = gql`
  subscription OnDeviceDisconnected {
    onDisconnected {
      id
      name
    }
  }
`;
