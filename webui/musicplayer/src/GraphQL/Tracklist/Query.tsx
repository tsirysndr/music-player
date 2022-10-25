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
          title
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
          title
        }
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
