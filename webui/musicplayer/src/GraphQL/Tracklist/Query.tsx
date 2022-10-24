import { gql } from "@apollo/client";
import { TRACK_FRAGMENT } from "../Fragments";

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
    currentlyPlayingSong {
      track {
        ...TrackFragment
        artists {
          name
        }
        album {
          title
        }
      }
      index
      isPlaying
      positionMs
    }
  }
  ${TRACK_FRAGMENT}
`;
