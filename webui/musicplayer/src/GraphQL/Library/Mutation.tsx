import gql from "graphql-tag";

export const LIKE_TRACK = gql`
  mutation LikeTrack($id: String!, $like: Boolean!) {
    likeTrack(id: $id, like: $like)
  }
`;
