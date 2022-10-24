import { gql } from "@apollo/client";
import { TRACK_FRAGMENT, ARTIST_FRAGMENT } from "../Fragments";

export const GET_CURRENTLY_PLAYING = gql`
  query CurrentlyPlayingSong {
    currentlyPlayingSong {
      track {
        ...TrackFragment
        artists {
          ...ArtistFragment
        }
      }
      isPlaying
      positionMs
    }
  }
  ${TRACK_FRAGMENT}
  ${ARTIST_FRAGMENT}
`;
