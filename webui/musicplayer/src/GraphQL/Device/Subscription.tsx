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
    }
  }
`;
