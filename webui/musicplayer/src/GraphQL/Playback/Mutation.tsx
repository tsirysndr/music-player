import gql from "graphql-tag";

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

export const SEEK = gql`
  mutation Seek($position: Int!) {
    seek(position: $position)
  }
`;
