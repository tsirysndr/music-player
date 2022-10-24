import { gql } from "@apollo/client";

export const GET_TRACKLIST = gql`
  query GetTracklist {
    tracklistTracks {
      previousTracks {
        title
      }
      nextTracks {
        title
      }
    }
  }
`;
