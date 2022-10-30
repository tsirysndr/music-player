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
          cover
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
