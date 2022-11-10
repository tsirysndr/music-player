import { gql } from '@apollo/client';
import * as Apollo from '@apollo/client';
export type Maybe<T> = T | null;
export type InputMaybe<T> = Maybe<T>;
export type Exact<T extends { [key: string]: unknown }> = { [K in keyof T]: T[K] };
export type MakeOptional<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]?: Maybe<T[SubKey]> };
export type MakeMaybe<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]: Maybe<T[SubKey]> };
const defaultOptions = {} as const;
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: string;
  String: string;
  Boolean: boolean;
  Int: number;
  Float: number;
};

export type Album = {
  __typename?: 'Album';
  artist: Scalars['String'];
  cover?: Maybe<Scalars['String']>;
  genres: Array<Scalars['String']>;
  id: Scalars['String'];
  releaseDate: Scalars['String'];
  title: Scalars['String'];
  tracks: Array<Track>;
  year?: Maybe<Scalars['Int']>;
};

export type Artist = {
  __typename?: 'Artist';
  albums: Array<Album>;
  bio: Scalars['String'];
  genres: Array<Scalars['String']>;
  id: Scalars['String'];
  images: Array<Scalars['String']>;
  name: Scalars['String'];
  picture: Scalars['String'];
  songs: Array<Track>;
  website: Scalars['String'];
};

export type CurrentlyPlayingSong = {
  __typename?: 'CurrentlyPlayingSong';
  index: Scalars['Int'];
  isPlaying: Scalars['Boolean'];
  positionMs: Scalars['Int'];
  track?: Maybe<Track>;
};

export type Mutation = {
  __typename?: 'Mutation';
  addTrack: Array<Track>;
  addTrackToPlaylist: Playlist;
  addTracks: Scalars['Boolean'];
  clearTracklist: Scalars['Boolean'];
  createPlaylist: Playlist;
  deletePlaylist: Scalars['Boolean'];
  next: Scalars['Boolean'];
  pause: Scalars['Boolean'];
  play: Scalars['Boolean'];
  playAlbum: Scalars['Boolean'];
  playArtistTracks: Scalars['Boolean'];
  playNext: Scalars['Boolean'];
  playPlaylist: Scalars['Boolean'];
  playTrackAt: Scalars['Boolean'];
  previous: Scalars['Boolean'];
  removeTrack: Scalars['Boolean'];
  removeTrackFromPlaylist: Playlist;
  renamePlaylist: Playlist;
  scan: Scalars['Boolean'];
  seek: Scalars['Boolean'];
  setMute: Scalars['Boolean'];
  setVolume: Scalars['Boolean'];
  shuffle: Scalars['Boolean'];
  stop: Scalars['Boolean'];
};


export type MutationAddTrackArgs = {
  track: TrackInput;
};


export type MutationAddTrackToPlaylistArgs = {
  id: Scalars['ID'];
  trackId: Scalars['ID'];
};


export type MutationAddTracksArgs = {
  tracks: Array<TrackInput>;
};


export type MutationCreatePlaylistArgs = {
  id: Scalars['ID'];
};


export type MutationDeletePlaylistArgs = {
  id: Scalars['ID'];
};


export type MutationPlayAlbumArgs = {
  id: Scalars['ID'];
  position?: InputMaybe<Scalars['Int']>;
  shuffle: Scalars['Boolean'];
};


export type MutationPlayArtistTracksArgs = {
  id: Scalars['ID'];
  position?: InputMaybe<Scalars['Int']>;
  shuffle: Scalars['Boolean'];
};


export type MutationPlayNextArgs = {
  id: Scalars['ID'];
};


export type MutationPlayPlaylistArgs = {
  id: Scalars['ID'];
  position?: InputMaybe<Scalars['Int']>;
  shuffle: Scalars['Boolean'];
};


export type MutationPlayTrackAtArgs = {
  position: Scalars['Int'];
};


export type MutationRemoveTrackArgs = {
  position: Scalars['Int'];
};


export type MutationRemoveTrackFromPlaylistArgs = {
  id: Scalars['ID'];
  trackId: Scalars['ID'];
};


export type MutationRenamePlaylistArgs = {
  id: Scalars['ID'];
  name: Scalars['String'];
};


export type MutationSeekArgs = {
  position: Scalars['Int'];
};


export type MutationSetMuteArgs = {
  mute: Scalars['Boolean'];
};


export type MutationSetVolumeArgs = {
  volume: Scalars['Int'];
};

export type Playlist = {
  __typename?: 'Playlist';
  id: Scalars['String'];
  name: Scalars['String'];
  tracks: Array<Track>;
};

export type Query = {
  __typename?: 'Query';
  album: Album;
  albums: Array<Album>;
  artist: Artist;
  artists: Array<Artist>;
  currentlyPlayingSong: CurrentlyPlayingSong;
  getNextTrack?: Maybe<Track>;
  getPlaybackState: Scalars['Boolean'];
  getPlaylistTracks: Scalars['Boolean'];
  getPreviousTrack?: Maybe<Track>;
  getRandom: Scalars['Boolean'];
  getRepeat: Scalars['Boolean'];
  getVolume: Scalars['Int'];
  playlist: Scalars['Boolean'];
  playlists: Scalars['Boolean'];
  search: Scalars['Boolean'];
  track: Track;
  tracklistTracks: Tracklist;
  tracks: Array<Track>;
};


export type QueryAlbumArgs = {
  id: Scalars['ID'];
};


export type QueryArtistArgs = {
  id: Scalars['ID'];
};


export type QueryGetPlaylistTracksArgs = {
  id: Scalars['ID'];
};


export type QueryTrackArgs = {
  id: Scalars['ID'];
};

export type Track = {
  __typename?: 'Track';
  album: Album;
  artist: Scalars['String'];
  artists: Array<Artist>;
  discNumber: Scalars['Int'];
  duration?: Maybe<Scalars['Float']>;
  id: Scalars['String'];
  title: Scalars['String'];
  trackNumber?: Maybe<Scalars['Int']>;
  uri: Scalars['String'];
};

export type TrackInput = {
  discNumber: Scalars['Int'];
  duration?: InputMaybe<Scalars['Float']>;
  id: Scalars['ID'];
  title: Scalars['String'];
  trackNumber?: InputMaybe<Scalars['Int']>;
  uri: Scalars['String'];
};

export type Tracklist = {
  __typename?: 'Tracklist';
  nextTracks: Array<Track>;
  previousTracks: Array<Track>;
};

export type AlbumFragmentFragment = { __typename?: 'Album', id: string, title: string, artist: string, year?: number | null, cover?: string | null };

export type ArtistFragmentFragment = { __typename?: 'Artist', id: string, name: string, picture: string };

export type TrackFragmentFragment = { __typename?: 'Track', id: string, trackNumber?: number | null, title: string, artist: string, duration?: number | null };

export type GetAlbumsQueryVariables = Exact<{ [key: string]: never; }>;


export type GetAlbumsQuery = { __typename?: 'Query', albums: Array<{ __typename?: 'Album', id: string, title: string, artist: string, year?: number | null, cover?: string | null, tracks: Array<{ __typename?: 'Track', id: string, trackNumber?: number | null, title: string, artist: string, duration?: number | null }> }> };

export type GetArtistsQueryVariables = Exact<{ [key: string]: never; }>;


export type GetArtistsQuery = { __typename?: 'Query', artists: Array<{ __typename?: 'Artist', id: string, name: string, picture: string }> };

export type GetTracksQueryVariables = Exact<{ [key: string]: never; }>;


export type GetTracksQuery = { __typename?: 'Query', tracks: Array<{ __typename?: 'Track', id: string, trackNumber?: number | null, title: string, artist: string, duration?: number | null, artists: Array<{ __typename?: 'Artist', id: string, name: string }>, album: { __typename?: 'Album', id: string, title: string, artist: string, year?: number | null, cover?: string | null } }> };

export type GetArtistQueryVariables = Exact<{
  id: Scalars['ID'];
}>;


export type GetArtistQuery = { __typename?: 'Query', artist: { __typename?: 'Artist', id: string, name: string, picture: string, songs: Array<{ __typename?: 'Track', id: string, title: string, artist: string, duration?: number | null, artists: Array<{ __typename?: 'Artist', id: string, name: string }>, album: { __typename?: 'Album', id: string, title: string, cover?: string | null } }>, albums: Array<{ __typename?: 'Album', id: string, title: string, artist: string, year?: number | null, cover?: string | null }> } };

export type GetAlbumQueryVariables = Exact<{
  id: Scalars['ID'];
}>;


export type GetAlbumQuery = { __typename?: 'Query', album: { __typename?: 'Album', id: string, title: string, artist: string, year?: number | null, cover?: string | null, tracks: Array<{ __typename?: 'Track', id: string, trackNumber?: number | null, title: string, artist: string, duration?: number | null, artists: Array<{ __typename?: 'Artist', id: string, name: string }> }> } };

export type NextMutationVariables = Exact<{ [key: string]: never; }>;


export type NextMutation = { __typename?: 'Mutation', next: boolean };

export type PauseMutationVariables = Exact<{ [key: string]: never; }>;


export type PauseMutation = { __typename?: 'Mutation', pause: boolean };

export type PlayMutationVariables = Exact<{ [key: string]: never; }>;


export type PlayMutation = { __typename?: 'Mutation', play: boolean };

export type PreviousMutationVariables = Exact<{ [key: string]: never; }>;


export type PreviousMutation = { __typename?: 'Mutation', previous: boolean };

export type CurrentlyPlayingSongQueryVariables = Exact<{ [key: string]: never; }>;


export type CurrentlyPlayingSongQuery = { __typename?: 'Query', currentlyPlayingSong: { __typename?: 'CurrentlyPlayingSong', index: number, isPlaying: boolean, positionMs: number, track?: { __typename?: 'Track', id: string, trackNumber?: number | null, title: string, artist: string, duration?: number | null, artists: Array<{ __typename?: 'Artist', id: string, name: string }>, album: { __typename?: 'Album', id: string, title: string } } | null } };

export type ClearTracklistMutationVariables = Exact<{ [key: string]: never; }>;


export type ClearTracklistMutation = { __typename?: 'Mutation', clearTracklist: boolean };

export type PlayAlbumMutationVariables = Exact<{
  albumId: Scalars['ID'];
  position?: InputMaybe<Scalars['Int']>;
  shuffle: Scalars['Boolean'];
}>;


export type PlayAlbumMutation = { __typename?: 'Mutation', playAlbum: boolean };

export type PlayArtistTracksMutationVariables = Exact<{
  artistId: Scalars['ID'];
  position?: InputMaybe<Scalars['Int']>;
  shuffle: Scalars['Boolean'];
}>;


export type PlayArtistTracksMutation = { __typename?: 'Mutation', playArtistTracks: boolean };

export type PlayTrackAtMutationVariables = Exact<{
  position: Scalars['Int'];
}>;


export type PlayTrackAtMutation = { __typename?: 'Mutation', playTrackAt: boolean };

export type RemoveTrackAtMutationVariables = Exact<{
  position: Scalars['Int'];
}>;


export type RemoveTrackAtMutation = { __typename?: 'Mutation', removeTrack: boolean };

export type PlayNextMutationVariables = Exact<{
  trackId: Scalars['ID'];
}>;


export type PlayNextMutation = { __typename?: 'Mutation', playNext: boolean };

export type GetTracklistQueryVariables = Exact<{ [key: string]: never; }>;


export type GetTracklistQuery = { __typename?: 'Query', tracklistTracks: { __typename?: 'Tracklist', previousTracks: Array<{ __typename?: 'Track', id: string, title: string, duration?: number | null, artists: Array<{ __typename?: 'Artist', id: string, name: string }>, album: { __typename?: 'Album', id: string, title: string, cover?: string | null } }>, nextTracks: Array<{ __typename?: 'Track', id: string, title: string, duration?: number | null, artists: Array<{ __typename?: 'Artist', id: string, name: string }>, album: { __typename?: 'Album', id: string, title: string, cover?: string | null } }> }, currentlyPlayingSong: { __typename?: 'CurrentlyPlayingSong', index: number, isPlaying: boolean, positionMs: number, track?: { __typename?: 'Track', id: string, trackNumber?: number | null, title: string, artist: string, duration?: number | null, artists: Array<{ __typename?: 'Artist', name: string }>, album: { __typename?: 'Album', title: string } } | null } };

export const AlbumFragmentFragmentDoc = gql`
    fragment AlbumFragment on Album {
  id
  title
  artist
  year
  cover
}
    `;
export const ArtistFragmentFragmentDoc = gql`
    fragment ArtistFragment on Artist {
  id
  name
  picture
}
    `;
export const TrackFragmentFragmentDoc = gql`
    fragment TrackFragment on Track {
  id
  trackNumber
  title
  artist
  duration
}
    `;
export const GetAlbumsDocument = gql`
    query GetAlbums {
  albums {
    ...AlbumFragment
    tracks {
      ...TrackFragment
    }
  }
}
    ${AlbumFragmentFragmentDoc}
${TrackFragmentFragmentDoc}`;

/**
 * __useGetAlbumsQuery__
 *
 * To run a query within a React component, call `useGetAlbumsQuery` and pass it any options that fit your needs.
 * When your component renders, `useGetAlbumsQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useGetAlbumsQuery({
 *   variables: {
 *   },
 * });
 */
export function useGetAlbumsQuery(baseOptions?: Apollo.QueryHookOptions<GetAlbumsQuery, GetAlbumsQueryVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useQuery<GetAlbumsQuery, GetAlbumsQueryVariables>(GetAlbumsDocument, options);
      }
export function useGetAlbumsLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<GetAlbumsQuery, GetAlbumsQueryVariables>) {
          const options = {...defaultOptions, ...baseOptions}
          return Apollo.useLazyQuery<GetAlbumsQuery, GetAlbumsQueryVariables>(GetAlbumsDocument, options);
        }
export type GetAlbumsQueryHookResult = ReturnType<typeof useGetAlbumsQuery>;
export type GetAlbumsLazyQueryHookResult = ReturnType<typeof useGetAlbumsLazyQuery>;
export type GetAlbumsQueryResult = Apollo.QueryResult<GetAlbumsQuery, GetAlbumsQueryVariables>;
export const GetArtistsDocument = gql`
    query GetArtists {
  artists {
    ...ArtistFragment
  }
}
    ${ArtistFragmentFragmentDoc}`;

/**
 * __useGetArtistsQuery__
 *
 * To run a query within a React component, call `useGetArtistsQuery` and pass it any options that fit your needs.
 * When your component renders, `useGetArtistsQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useGetArtistsQuery({
 *   variables: {
 *   },
 * });
 */
export function useGetArtistsQuery(baseOptions?: Apollo.QueryHookOptions<GetArtistsQuery, GetArtistsQueryVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useQuery<GetArtistsQuery, GetArtistsQueryVariables>(GetArtistsDocument, options);
      }
export function useGetArtistsLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<GetArtistsQuery, GetArtistsQueryVariables>) {
          const options = {...defaultOptions, ...baseOptions}
          return Apollo.useLazyQuery<GetArtistsQuery, GetArtistsQueryVariables>(GetArtistsDocument, options);
        }
export type GetArtistsQueryHookResult = ReturnType<typeof useGetArtistsQuery>;
export type GetArtistsLazyQueryHookResult = ReturnType<typeof useGetArtistsLazyQuery>;
export type GetArtistsQueryResult = Apollo.QueryResult<GetArtistsQuery, GetArtistsQueryVariables>;
export const GetTracksDocument = gql`
    query GetTracks {
  tracks {
    ...TrackFragment
    artists {
      id
      name
    }
    album {
      ...AlbumFragment
    }
  }
}
    ${TrackFragmentFragmentDoc}
${AlbumFragmentFragmentDoc}`;

/**
 * __useGetTracksQuery__
 *
 * To run a query within a React component, call `useGetTracksQuery` and pass it any options that fit your needs.
 * When your component renders, `useGetTracksQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useGetTracksQuery({
 *   variables: {
 *   },
 * });
 */
export function useGetTracksQuery(baseOptions?: Apollo.QueryHookOptions<GetTracksQuery, GetTracksQueryVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useQuery<GetTracksQuery, GetTracksQueryVariables>(GetTracksDocument, options);
      }
export function useGetTracksLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<GetTracksQuery, GetTracksQueryVariables>) {
          const options = {...defaultOptions, ...baseOptions}
          return Apollo.useLazyQuery<GetTracksQuery, GetTracksQueryVariables>(GetTracksDocument, options);
        }
export type GetTracksQueryHookResult = ReturnType<typeof useGetTracksQuery>;
export type GetTracksLazyQueryHookResult = ReturnType<typeof useGetTracksLazyQuery>;
export type GetTracksQueryResult = Apollo.QueryResult<GetTracksQuery, GetTracksQueryVariables>;
export const GetArtistDocument = gql`
    query GetArtist($id: ID!) {
  artist(id: $id) {
    id
    name
    picture
    songs {
      id
      title
      artist
      artists {
        id
        name
      }
      album {
        id
        title
        cover
      }
      duration
    }
    albums {
      ...AlbumFragment
    }
  }
}
    ${AlbumFragmentFragmentDoc}`;

/**
 * __useGetArtistQuery__
 *
 * To run a query within a React component, call `useGetArtistQuery` and pass it any options that fit your needs.
 * When your component renders, `useGetArtistQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useGetArtistQuery({
 *   variables: {
 *      id: // value for 'id'
 *   },
 * });
 */
export function useGetArtistQuery(baseOptions: Apollo.QueryHookOptions<GetArtistQuery, GetArtistQueryVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useQuery<GetArtistQuery, GetArtistQueryVariables>(GetArtistDocument, options);
      }
export function useGetArtistLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<GetArtistQuery, GetArtistQueryVariables>) {
          const options = {...defaultOptions, ...baseOptions}
          return Apollo.useLazyQuery<GetArtistQuery, GetArtistQueryVariables>(GetArtistDocument, options);
        }
export type GetArtistQueryHookResult = ReturnType<typeof useGetArtistQuery>;
export type GetArtistLazyQueryHookResult = ReturnType<typeof useGetArtistLazyQuery>;
export type GetArtistQueryResult = Apollo.QueryResult<GetArtistQuery, GetArtistQueryVariables>;
export const GetAlbumDocument = gql`
    query GetAlbum($id: ID!) {
  album(id: $id) {
    ...AlbumFragment
    tracks {
      id
      trackNumber
      title
      artist
      artists {
        id
        name
      }
      duration
    }
  }
}
    ${AlbumFragmentFragmentDoc}`;

/**
 * __useGetAlbumQuery__
 *
 * To run a query within a React component, call `useGetAlbumQuery` and pass it any options that fit your needs.
 * When your component renders, `useGetAlbumQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useGetAlbumQuery({
 *   variables: {
 *      id: // value for 'id'
 *   },
 * });
 */
export function useGetAlbumQuery(baseOptions: Apollo.QueryHookOptions<GetAlbumQuery, GetAlbumQueryVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useQuery<GetAlbumQuery, GetAlbumQueryVariables>(GetAlbumDocument, options);
      }
export function useGetAlbumLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<GetAlbumQuery, GetAlbumQueryVariables>) {
          const options = {...defaultOptions, ...baseOptions}
          return Apollo.useLazyQuery<GetAlbumQuery, GetAlbumQueryVariables>(GetAlbumDocument, options);
        }
export type GetAlbumQueryHookResult = ReturnType<typeof useGetAlbumQuery>;
export type GetAlbumLazyQueryHookResult = ReturnType<typeof useGetAlbumLazyQuery>;
export type GetAlbumQueryResult = Apollo.QueryResult<GetAlbumQuery, GetAlbumQueryVariables>;
export const NextDocument = gql`
    mutation Next {
  next
}
    `;
export type NextMutationFn = Apollo.MutationFunction<NextMutation, NextMutationVariables>;

/**
 * __useNextMutation__
 *
 * To run a mutation, you first call `useNextMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `useNextMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [nextMutation, { data, loading, error }] = useNextMutation({
 *   variables: {
 *   },
 * });
 */
export function useNextMutation(baseOptions?: Apollo.MutationHookOptions<NextMutation, NextMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<NextMutation, NextMutationVariables>(NextDocument, options);
      }
export type NextMutationHookResult = ReturnType<typeof useNextMutation>;
export type NextMutationResult = Apollo.MutationResult<NextMutation>;
export type NextMutationOptions = Apollo.BaseMutationOptions<NextMutation, NextMutationVariables>;
export const PauseDocument = gql`
    mutation Pause {
  pause
}
    `;
export type PauseMutationFn = Apollo.MutationFunction<PauseMutation, PauseMutationVariables>;

/**
 * __usePauseMutation__
 *
 * To run a mutation, you first call `usePauseMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `usePauseMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [pauseMutation, { data, loading, error }] = usePauseMutation({
 *   variables: {
 *   },
 * });
 */
export function usePauseMutation(baseOptions?: Apollo.MutationHookOptions<PauseMutation, PauseMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<PauseMutation, PauseMutationVariables>(PauseDocument, options);
      }
export type PauseMutationHookResult = ReturnType<typeof usePauseMutation>;
export type PauseMutationResult = Apollo.MutationResult<PauseMutation>;
export type PauseMutationOptions = Apollo.BaseMutationOptions<PauseMutation, PauseMutationVariables>;
export const PlayDocument = gql`
    mutation Play {
  play
}
    `;
export type PlayMutationFn = Apollo.MutationFunction<PlayMutation, PlayMutationVariables>;

/**
 * __usePlayMutation__
 *
 * To run a mutation, you first call `usePlayMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `usePlayMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [playMutation, { data, loading, error }] = usePlayMutation({
 *   variables: {
 *   },
 * });
 */
export function usePlayMutation(baseOptions?: Apollo.MutationHookOptions<PlayMutation, PlayMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<PlayMutation, PlayMutationVariables>(PlayDocument, options);
      }
export type PlayMutationHookResult = ReturnType<typeof usePlayMutation>;
export type PlayMutationResult = Apollo.MutationResult<PlayMutation>;
export type PlayMutationOptions = Apollo.BaseMutationOptions<PlayMutation, PlayMutationVariables>;
export const PreviousDocument = gql`
    mutation Previous {
  previous
}
    `;
export type PreviousMutationFn = Apollo.MutationFunction<PreviousMutation, PreviousMutationVariables>;

/**
 * __usePreviousMutation__
 *
 * To run a mutation, you first call `usePreviousMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `usePreviousMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [previousMutation, { data, loading, error }] = usePreviousMutation({
 *   variables: {
 *   },
 * });
 */
export function usePreviousMutation(baseOptions?: Apollo.MutationHookOptions<PreviousMutation, PreviousMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<PreviousMutation, PreviousMutationVariables>(PreviousDocument, options);
      }
export type PreviousMutationHookResult = ReturnType<typeof usePreviousMutation>;
export type PreviousMutationResult = Apollo.MutationResult<PreviousMutation>;
export type PreviousMutationOptions = Apollo.BaseMutationOptions<PreviousMutation, PreviousMutationVariables>;
export const CurrentlyPlayingSongDocument = gql`
    query CurrentlyPlayingSong {
  currentlyPlayingSong {
    track {
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
    index
    isPlaying
    positionMs
  }
}
    ${TrackFragmentFragmentDoc}`;

/**
 * __useCurrentlyPlayingSongQuery__
 *
 * To run a query within a React component, call `useCurrentlyPlayingSongQuery` and pass it any options that fit your needs.
 * When your component renders, `useCurrentlyPlayingSongQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useCurrentlyPlayingSongQuery({
 *   variables: {
 *   },
 * });
 */
export function useCurrentlyPlayingSongQuery(baseOptions?: Apollo.QueryHookOptions<CurrentlyPlayingSongQuery, CurrentlyPlayingSongQueryVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useQuery<CurrentlyPlayingSongQuery, CurrentlyPlayingSongQueryVariables>(CurrentlyPlayingSongDocument, options);
      }
export function useCurrentlyPlayingSongLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<CurrentlyPlayingSongQuery, CurrentlyPlayingSongQueryVariables>) {
          const options = {...defaultOptions, ...baseOptions}
          return Apollo.useLazyQuery<CurrentlyPlayingSongQuery, CurrentlyPlayingSongQueryVariables>(CurrentlyPlayingSongDocument, options);
        }
export type CurrentlyPlayingSongQueryHookResult = ReturnType<typeof useCurrentlyPlayingSongQuery>;
export type CurrentlyPlayingSongLazyQueryHookResult = ReturnType<typeof useCurrentlyPlayingSongLazyQuery>;
export type CurrentlyPlayingSongQueryResult = Apollo.QueryResult<CurrentlyPlayingSongQuery, CurrentlyPlayingSongQueryVariables>;
export const ClearTracklistDocument = gql`
    mutation ClearTracklist {
  clearTracklist
}
    `;
export type ClearTracklistMutationFn = Apollo.MutationFunction<ClearTracklistMutation, ClearTracklistMutationVariables>;

/**
 * __useClearTracklistMutation__
 *
 * To run a mutation, you first call `useClearTracklistMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `useClearTracklistMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [clearTracklistMutation, { data, loading, error }] = useClearTracklistMutation({
 *   variables: {
 *   },
 * });
 */
export function useClearTracklistMutation(baseOptions?: Apollo.MutationHookOptions<ClearTracklistMutation, ClearTracklistMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<ClearTracklistMutation, ClearTracklistMutationVariables>(ClearTracklistDocument, options);
      }
export type ClearTracklistMutationHookResult = ReturnType<typeof useClearTracklistMutation>;
export type ClearTracklistMutationResult = Apollo.MutationResult<ClearTracklistMutation>;
export type ClearTracklistMutationOptions = Apollo.BaseMutationOptions<ClearTracklistMutation, ClearTracklistMutationVariables>;
export const PlayAlbumDocument = gql`
    mutation PlayAlbum($albumId: ID!, $position: Int, $shuffle: Boolean!) {
  playAlbum(id: $albumId, position: $position, shuffle: $shuffle)
}
    `;
export type PlayAlbumMutationFn = Apollo.MutationFunction<PlayAlbumMutation, PlayAlbumMutationVariables>;

/**
 * __usePlayAlbumMutation__
 *
 * To run a mutation, you first call `usePlayAlbumMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `usePlayAlbumMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [playAlbumMutation, { data, loading, error }] = usePlayAlbumMutation({
 *   variables: {
 *      albumId: // value for 'albumId'
 *      position: // value for 'position'
 *      shuffle: // value for 'shuffle'
 *   },
 * });
 */
export function usePlayAlbumMutation(baseOptions?: Apollo.MutationHookOptions<PlayAlbumMutation, PlayAlbumMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<PlayAlbumMutation, PlayAlbumMutationVariables>(PlayAlbumDocument, options);
      }
export type PlayAlbumMutationHookResult = ReturnType<typeof usePlayAlbumMutation>;
export type PlayAlbumMutationResult = Apollo.MutationResult<PlayAlbumMutation>;
export type PlayAlbumMutationOptions = Apollo.BaseMutationOptions<PlayAlbumMutation, PlayAlbumMutationVariables>;
export const PlayArtistTracksDocument = gql`
    mutation PlayArtistTracks($artistId: ID!, $position: Int, $shuffle: Boolean!) {
  playArtistTracks(id: $artistId, position: $position, shuffle: $shuffle)
}
    `;
export type PlayArtistTracksMutationFn = Apollo.MutationFunction<PlayArtistTracksMutation, PlayArtistTracksMutationVariables>;

/**
 * __usePlayArtistTracksMutation__
 *
 * To run a mutation, you first call `usePlayArtistTracksMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `usePlayArtistTracksMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [playArtistTracksMutation, { data, loading, error }] = usePlayArtistTracksMutation({
 *   variables: {
 *      artistId: // value for 'artistId'
 *      position: // value for 'position'
 *      shuffle: // value for 'shuffle'
 *   },
 * });
 */
export function usePlayArtistTracksMutation(baseOptions?: Apollo.MutationHookOptions<PlayArtistTracksMutation, PlayArtistTracksMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<PlayArtistTracksMutation, PlayArtistTracksMutationVariables>(PlayArtistTracksDocument, options);
      }
export type PlayArtistTracksMutationHookResult = ReturnType<typeof usePlayArtistTracksMutation>;
export type PlayArtistTracksMutationResult = Apollo.MutationResult<PlayArtistTracksMutation>;
export type PlayArtistTracksMutationOptions = Apollo.BaseMutationOptions<PlayArtistTracksMutation, PlayArtistTracksMutationVariables>;
export const PlayTrackAtDocument = gql`
    mutation PlayTrackAt($position: Int!) {
  playTrackAt(position: $position)
}
    `;
export type PlayTrackAtMutationFn = Apollo.MutationFunction<PlayTrackAtMutation, PlayTrackAtMutationVariables>;

/**
 * __usePlayTrackAtMutation__
 *
 * To run a mutation, you first call `usePlayTrackAtMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `usePlayTrackAtMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [playTrackAtMutation, { data, loading, error }] = usePlayTrackAtMutation({
 *   variables: {
 *      position: // value for 'position'
 *   },
 * });
 */
export function usePlayTrackAtMutation(baseOptions?: Apollo.MutationHookOptions<PlayTrackAtMutation, PlayTrackAtMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<PlayTrackAtMutation, PlayTrackAtMutationVariables>(PlayTrackAtDocument, options);
      }
export type PlayTrackAtMutationHookResult = ReturnType<typeof usePlayTrackAtMutation>;
export type PlayTrackAtMutationResult = Apollo.MutationResult<PlayTrackAtMutation>;
export type PlayTrackAtMutationOptions = Apollo.BaseMutationOptions<PlayTrackAtMutation, PlayTrackAtMutationVariables>;
export const RemoveTrackAtDocument = gql`
    mutation RemoveTrackAt($position: Int!) {
  removeTrack(position: $position)
}
    `;
export type RemoveTrackAtMutationFn = Apollo.MutationFunction<RemoveTrackAtMutation, RemoveTrackAtMutationVariables>;

/**
 * __useRemoveTrackAtMutation__
 *
 * To run a mutation, you first call `useRemoveTrackAtMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `useRemoveTrackAtMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [removeTrackAtMutation, { data, loading, error }] = useRemoveTrackAtMutation({
 *   variables: {
 *      position: // value for 'position'
 *   },
 * });
 */
export function useRemoveTrackAtMutation(baseOptions?: Apollo.MutationHookOptions<RemoveTrackAtMutation, RemoveTrackAtMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<RemoveTrackAtMutation, RemoveTrackAtMutationVariables>(RemoveTrackAtDocument, options);
      }
export type RemoveTrackAtMutationHookResult = ReturnType<typeof useRemoveTrackAtMutation>;
export type RemoveTrackAtMutationResult = Apollo.MutationResult<RemoveTrackAtMutation>;
export type RemoveTrackAtMutationOptions = Apollo.BaseMutationOptions<RemoveTrackAtMutation, RemoveTrackAtMutationVariables>;
export const PlayNextDocument = gql`
    mutation PlayNext($trackId: ID!) {
  playNext(id: $trackId)
}
    `;
export type PlayNextMutationFn = Apollo.MutationFunction<PlayNextMutation, PlayNextMutationVariables>;

/**
 * __usePlayNextMutation__
 *
 * To run a mutation, you first call `usePlayNextMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `usePlayNextMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [playNextMutation, { data, loading, error }] = usePlayNextMutation({
 *   variables: {
 *      trackId: // value for 'trackId'
 *   },
 * });
 */
export function usePlayNextMutation(baseOptions?: Apollo.MutationHookOptions<PlayNextMutation, PlayNextMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<PlayNextMutation, PlayNextMutationVariables>(PlayNextDocument, options);
      }
export type PlayNextMutationHookResult = ReturnType<typeof usePlayNextMutation>;
export type PlayNextMutationResult = Apollo.MutationResult<PlayNextMutation>;
export type PlayNextMutationOptions = Apollo.BaseMutationOptions<PlayNextMutation, PlayNextMutationVariables>;
export const GetTracklistDocument = gql`
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
    ${TrackFragmentFragmentDoc}`;

/**
 * __useGetTracklistQuery__
 *
 * To run a query within a React component, call `useGetTracklistQuery` and pass it any options that fit your needs.
 * When your component renders, `useGetTracklistQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useGetTracklistQuery({
 *   variables: {
 *   },
 * });
 */
export function useGetTracklistQuery(baseOptions?: Apollo.QueryHookOptions<GetTracklistQuery, GetTracklistQueryVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useQuery<GetTracklistQuery, GetTracklistQueryVariables>(GetTracklistDocument, options);
      }
export function useGetTracklistLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<GetTracklistQuery, GetTracklistQueryVariables>) {
          const options = {...defaultOptions, ...baseOptions}
          return Apollo.useLazyQuery<GetTracklistQuery, GetTracklistQueryVariables>(GetTracklistDocument, options);
        }
export type GetTracklistQueryHookResult = ReturnType<typeof useGetTracklistQuery>;
export type GetTracklistLazyQueryHookResult = ReturnType<typeof useGetTracklistLazyQuery>;
export type GetTracklistQueryResult = Apollo.QueryResult<GetTracklistQuery, GetTracklistQueryVariables>;