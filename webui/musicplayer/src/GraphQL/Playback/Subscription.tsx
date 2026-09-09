import gql from "graphql-tag";
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
        cover
      }
    }
  }
  ${TRACK_FRAGMENT}
`;

/**
 * Output levels for the VU meter, pushed at 20 Hz.
 *
 * A subscription rather than a poll: a meter wants tens of updates a second,
 * and the now-playing poll is far slower than that.
 */
export const ON_LEVELS = gql`
  subscription OnLevels {
    levels {
      left
      right
      lowLeft
      lowRight
    }
  }
`;
