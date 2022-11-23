import { gql } from "@apollo/client";
import { TRACK_FRAGMENT } from "../Fragments";

export const PLAYER_STATE = gql`
  subscription PlayerStateChanged {
    playerState {
      isPlaying
    }
  }
`;

export const TRACK_TIME_POSITION = gql`
  subscription TrackTimePositionChanged {
    trackTimePosition {
      positionMs
    }
  }
`;

export const CURRENTLY_PLAYING_SONG = gql`
  subscription CurrentlyPlayingSongChanged {
    currentlyPlayingSong {
      ...TrackFragment
      artists {
        id
        name
      }
      album {
        id
        title
      }
    }
  }
  ${TRACK_FRAGMENT}
`;
