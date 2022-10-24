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
  cover: Scalars['String'];
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


export type MutationPlayNextArgs = {
  id: Scalars['ID'];
};


export type MutationPlayPlaylistArgs = {
  id: Scalars['ID'];
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

export type AlbumFragmentFragment = { __typename?: 'Album', id: string, title: string, artist: string, year?: number | null, cover: string };

export type ArtistFragmentFragment = { __typename?: 'Artist', id: string, name: string };

export type TrackFragmentFragment = { __typename?: 'Track', id: string, trackNumber?: number | null, title: string, duration?: number | null };

export type GetAlbumsQueryVariables = Exact<{ [key: string]: never; }>;


export type GetAlbumsQuery = { __typename?: 'Query', albums: Array<{ __typename?: 'Album', id: string, title: string, artist: string, year?: number | null, cover: string, tracks: Array<{ __typename?: 'Track', id: string, trackNumber?: number | null, title: string, duration?: number | null }> }> };

export type GetArtistsQueryVariables = Exact<{ [key: string]: never; }>;


export type GetArtistsQuery = { __typename?: 'Query', artists: Array<{ __typename?: 'Artist', id: string, name: string }> };

export type GetTracksQueryVariables = Exact<{ [key: string]: never; }>;


export type GetTracksQuery = { __typename?: 'Query', tracks: Array<{ __typename?: 'Track', id: string, trackNumber?: number | null, title: string, duration?: number | null, artists: Array<{ __typename?: 'Artist', id: string, name: string }>, album: { __typename?: 'Album', id: string, title: string, artist: string, year?: number | null, cover: string } }> };

export type NextMutationVariables = Exact<{ [key: string]: never; }>;


export type NextMutation = { __typename?: 'Mutation', next: boolean };

export type PauseMutationVariables = Exact<{ [key: string]: never; }>;


export type PauseMutation = { __typename?: 'Mutation', pause: boolean };

export type PlayMutationVariables = Exact<{ [key: string]: never; }>;


export type PlayMutation = { __typename?: 'Mutation', play: boolean };

export type PreviousMutationVariables = Exact<{ [key: string]: never; }>;


export type PreviousMutation = { __typename?: 'Mutation', previous: boolean };

export type CurrentlyPlayingSongQueryVariables = Exact<{ [key: string]: never; }>;


export type CurrentlyPlayingSongQuery = { __typename?: 'Query', currentlyPlayingSong: { __typename?: 'CurrentlyPlayingSong', index: number, isPlaying: boolean, positionMs: number, track?: { __typename?: 'Track', id: string, trackNumber?: number | null, title: string, duration?: number | null, artists: Array<{ __typename?: 'Artist', name: string }>, album: { __typename?: 'Album', title: string } } | null } };

export type GetTracklistQueryVariables = Exact<{ [key: string]: never; }>;


export type GetTracklistQuery = { __typename?: 'Query', tracklistTracks: { __typename?: 'Tracklist', previousTracks: Array<{ __typename?: 'Track', title: string }>, nextTracks: Array<{ __typename?: 'Track', title: string }> }, currentlyPlayingSong: { __typename?: 'CurrentlyPlayingSong', index: number, isPlaying: boolean, positionMs: number, track?: { __typename?: 'Track', id: string, trackNumber?: number | null, title: string, duration?: number | null, artists: Array<{ __typename?: 'Artist', name: string }>, album: { __typename?: 'Album', title: string } } | null } };

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
}
    `;
export const TrackFragmentFragmentDoc = gql`
    fragment TrackFragment on Track {
  id
  trackNumber
  title
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
export const GetTracklistDocument = gql`
    query GetTracklist {
  tracklistTracks {
    previousTracks {
      title
    }
    nextTracks {
      title
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