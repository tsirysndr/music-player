import {gql} from '@apollo/client';

export const NEXT = gql`
  mutation Next {
    next
  }
`;

export const PAUSE = gql`
  mutation Pause {
    pause
  }
`;

export const PLAY = gql`
  mutation Play {
    play
  }
`;

export const PREVIOUS = gql`
  mutation Previous {
    previous
  }
`;
