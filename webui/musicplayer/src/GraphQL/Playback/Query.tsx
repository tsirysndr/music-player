import { gql } from "@apollo/client";

export const GET_CURRENTLY_PLAYING = gql`
  query CurrentlyPlayingSong {
    currentlyPlayingSong {
      id
      trackNumber
      title
      artist
      duration
      album {
        id
        title
        artist
        year
        cover
      }
      artist
      artists {
        id
        name
        picture
      }
    }
  }
`;

export const GET_PLAYER_STATE = gql`
  query GetPlayerState {
    getPlayerState {
      isPlaying
      index
    }
  }
`;

export const GET_PLAYBACK_PROGRESS = gql`
  query GetPlaybackProgress {
    playbackProgress
  }
`;
