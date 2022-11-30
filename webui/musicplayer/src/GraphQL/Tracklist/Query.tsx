import { gql } from "@apollo/client";
import { TRACK_FRAGMENT } from "../Fragments";

export const GET_TRACKLIST = gql`
  query GetTracklist {
    tracklistTracks {
      previousTracks {
        id
        title
        duration
        artists {
          id
          name
        }
        album {
          id
          title
          artist
          cover
        }
      }
      nextTracks {
        id
        title
        duration
        artists {
          id
          name
        }
        album {
          id
          title
          artist
          cover
        }
      }
    }
    currentlyPlayingSong {
      ...TrackFragment
    }
  }
  ${TRACK_FRAGMENT}
`;
