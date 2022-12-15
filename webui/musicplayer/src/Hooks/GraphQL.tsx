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

export enum App {
  MusicPlayer = 'MUSIC_PLAYER',
  Xbmc = 'XBMC'
}

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

export type Device = {
  __typename?: 'Device';
  app: Scalars['String'];
  host: Scalars['String'];
  id: Scalars['String'];
  isConnected: Scalars['Boolean'];
  name: Scalars['String'];
  port: Scalars['Int'];
  service: Scalars['String'];
};

export type Folder = {
  __typename?: 'Folder';
  id: Scalars['String'];
  name: Scalars['String'];
  playlists: Array<Playlist>;
};

export type FolderChanged = {
  __typename?: 'FolderChanged';
  folder: Folder;
  mutationType: MutationType;
  playlist?: Maybe<Playlist>;
};

export type Mutation = {
  __typename?: 'Mutation';
  addTrack: Array<Track>;
  addTrackToPlaylist: Playlist;
  addTracks: Scalars['Boolean'];
  clearTracklist: Scalars['Boolean'];
  connectToDevice: Device;
  createFolder: Folder;
  createPlaylist: Playlist;
  deleteFolder: Folder;
  deletePlaylist: Playlist;
  disconnectFromDevice: Device;
  movePlaylistToFolder: Folder;
  movePlaylistsToFolder: Folder;
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
  renameFolder: Folder;
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


export type MutationConnectToDeviceArgs = {
  id: Scalars['ID'];
};


export type MutationCreateFolderArgs = {
  name: Scalars['String'];
};


export type MutationCreatePlaylistArgs = {
  description?: InputMaybe<Scalars['String']>;
  folderId?: InputMaybe<Scalars['ID']>;
  name: Scalars['String'];
};


export type MutationDeleteFolderArgs = {
  id: Scalars['ID'];
};


export type MutationDeletePlaylistArgs = {
  id: Scalars['ID'];
};


export type MutationDisconnectFromDeviceArgs = {
  id: Scalars['ID'];
};


export type MutationMovePlaylistToFolderArgs = {
  folderId: Scalars['ID'];
  id: Scalars['ID'];
};


export type MutationMovePlaylistsToFolderArgs = {
  folderId: Scalars['ID'];
  ids: Array<Scalars['ID']>;
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
  position: Scalars['Int'];
};


export type MutationRenameFolderArgs = {
  id: Scalars['ID'];
  name: Scalars['String'];
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

export enum MutationType {
  Cleared = 'CLEARED',
  Created = 'CREATED',
  Deleted = 'DELETED',
  Moved = 'MOVED',
  Renamed = 'RENAMED',
  Updated = 'UPDATED'
}

export type PlayerState = {
  __typename?: 'PlayerState';
  index: Scalars['Int'];
  isPlaying: Scalars['Boolean'];
  positionMs: Scalars['Int'];
};

export type Playlist = {
  __typename?: 'Playlist';
  description?: Maybe<Scalars['String']>;
  id: Scalars['String'];
  name: Scalars['String'];
  tracks: Array<Track>;
};

export type PlaylistChanged = {
  __typename?: 'PlaylistChanged';
  mutationType: MutationType;
  playlist: Playlist;
  track?: Maybe<Track>;
};

export type PositionMilliseconds = {
  __typename?: 'PositionMilliseconds';
  positionMs: Scalars['Int'];
};

export type Query = {
  __typename?: 'Query';
  album: Album;
  albums: Array<Album>;
  artist: Artist;
  artists: Array<Artist>;
  connectedDevice: Device;
  currentlyPlayingSong: CurrentlyPlayingSong;
  folder: Folder;
  folders: Array<Folder>;
  getNextTrack?: Maybe<Track>;
  getPlayerState: PlayerState;
  getPreviousTrack?: Maybe<Track>;
  getRandom: Scalars['Boolean'];
  getRepeat: Scalars['Boolean'];
  getVolume: Scalars['Int'];
  listDevices: Array<Device>;
  mainPlaylists: Array<Playlist>;
  playlist: Playlist;
  playlists: Array<Playlist>;
  recentPlaylists: Array<Playlist>;
  search: SearchResult;
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


export type QueryFolderArgs = {
  id: Scalars['ID'];
};


export type QueryListDevicesArgs = {
  filter?: InputMaybe<App>;
};


export type QueryPlaylistArgs = {
  id: Scalars['ID'];
};


export type QuerySearchArgs = {
  keyword: Scalars['String'];
};


export type QueryTrackArgs = {
  id: Scalars['ID'];
};

export type SearchResult = {
  __typename?: 'SearchResult';
  albums: Array<Album>;
  artists: Array<Artist>;
  tracks: Array<Track>;
};

export type Subscription = {
  __typename?: 'Subscription';
  currentlyPlayingSong: Track;
  folder: FolderChanged;
  folders: Array<Folder>;
  onNewDevice: Device;
  playerState: PlayerState;
  playlist: PlaylistChanged;
  playlists: Array<Playlist>;
  trackTimePosition: PositionMilliseconds;
  tracklist: TracklistChanged;
};


export type SubscriptionFolderArgs = {
  id: Scalars['ID'];
};


export type SubscriptionPlaylistArgs = {
  id: Scalars['ID'];
};


export type SubscriptionTracklistArgs = {
  id: Scalars['ID'];
};

export type Track = {
  __typename?: 'Track';
  album: Album;
  albumId: Scalars['String'];
  albumTitle: Scalars['String'];
  artist: Scalars['String'];
  artistId: Scalars['String'];
  artists: Array<Artist>;
  cover?: Maybe<Scalars['String']>;
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

export type TracklistChanged = {
  __typename?: 'TracklistChanged';
  mutationType: MutationType;
  track?: Maybe<Track>;
  tracklist: Tracklist;
};

export type ListDevicesQueryVariables = Exact<{ [key: string]: never; }>;


export type ListDevicesQuery = { __typename?: 'Query', listDevices: Array<{ __typename?: 'Device', id: string, app: string, name: string, service: string, host: string, port: number, isConnected: boolean }> };

export type OnNewDeviceSubscriptionVariables = Exact<{ [key: string]: never; }>;


export type OnNewDeviceSubscription = { __typename?: 'Subscription', onNewDevice: { __typename?: 'Device', id: string, app: string, name: string, service: string, host: string, port: number, isConnected: boolean } };

export type AlbumFragmentFragment = { __typename?: 'Album', id: string, title: string, artist: string, year?: number | null, cover?: string | null };

export type ArtistFragmentFragment = { __typename?: 'Artist', id: string, name: string, picture: string };

export type TrackFragmentFragment = { __typename?: 'Track', id: string, trackNumber?: number | null, title: string, artist: string, duration?: number | null };

export type PlaylistFragmentFragment = { __typename?: 'Playlist', id: string, name: string, description?: string | null, tracks: Array<{ __typename?: 'Track', id: string, title: string, albumTitle: string, artist: string, artistId: string, albumId: string, cover?: string | null, duration?: number | null }> };

export type FolderFragmentFragment = { __typename?: 'Folder', id: string, name: string, playlists: Array<{ __typename?: 'Playlist', id: string, name: string, description?: string | null }> };

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

export type SearchQueryVariables = Exact<{
  keyword: Scalars['String'];
}>;


export type SearchQuery = { __typename?: 'Query', search: { __typename?: 'SearchResult', artists: Array<{ __typename?: 'Artist', id: string, name: string, picture: string }>, albums: Array<{ __typename?: 'Album', id: string, title: string, artist: string, cover?: string | null }>, tracks: Array<{ __typename?: 'Track', id: string, title: string, artist: string, duration?: number | null, cover?: string | null, artistId: string, albumId: string, albumTitle: string }> } };

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

export type PlayerStateChangedSubscriptionVariables = Exact<{ [key: string]: never; }>;


export type PlayerStateChangedSubscription = { __typename?: 'Subscription', playerState: { __typename?: 'PlayerState', isPlaying: boolean } };

export type TrackTimePositionChangedSubscriptionVariables = Exact<{ [key: string]: never; }>;


export type TrackTimePositionChangedSubscription = { __typename?: 'Subscription', trackTimePosition: { __typename?: 'PositionMilliseconds', positionMs: number } };

export type CurrentlyPlayingSongChangedSubscriptionVariables = Exact<{ [key: string]: never; }>;


export type CurrentlyPlayingSongChangedSubscription = { __typename?: 'Subscription', currentlyPlayingSong: { __typename?: 'Track', id: string, trackNumber?: number | null, title: string, artist: string, duration?: number | null, artists: Array<{ __typename?: 'Artist', id: string, name: string }>, album: { __typename?: 'Album', id: string, title: string } } };

export type CreatePlaylistMutationVariables = Exact<{
  name: Scalars['String'];
  description?: InputMaybe<Scalars['String']>;
}>;


export type CreatePlaylistMutation = { __typename?: 'Mutation', createPlaylist: { __typename?: 'Playlist', id: string, name: string, description?: string | null } };

export type DeletePlaylistMutationVariables = Exact<{
  id: Scalars['ID'];
}>;


export type DeletePlaylistMutation = { __typename?: 'Mutation', deletePlaylist: { __typename?: 'Playlist', id: string } };

export type AddTrackToPlaylistMutationVariables = Exact<{
  playlistId: Scalars['ID'];
  trackId: Scalars['ID'];
}>;


export type AddTrackToPlaylistMutation = { __typename?: 'Mutation', addTrackToPlaylist: { __typename?: 'Playlist', id: string, name: string, description?: string | null, tracks: Array<{ __typename?: 'Track', id: string, title: string, albumTitle: string, artist: string, artistId: string, albumId: string, cover?: string | null, duration?: number | null }> } };

export type RemoveTrackFromPlaylistMutationVariables = Exact<{
  playlistId: Scalars['ID'];
  position: Scalars['Int'];
}>;


export type RemoveTrackFromPlaylistMutation = { __typename?: 'Mutation', removeTrackFromPlaylist: { __typename?: 'Playlist', id: string, name: string, description?: string | null, tracks: Array<{ __typename?: 'Track', id: string, title: string, albumTitle: string, artist: string, artistId: string, albumId: string, cover?: string | null, duration?: number | null }> } };

export type RenamePlaylistMutationVariables = Exact<{
  id: Scalars['ID'];
  name: Scalars['String'];
}>;


export type RenamePlaylistMutation = { __typename?: 'Mutation', renamePlaylist: { __typename?: 'Playlist', id: string, name: string } };

export type CreateFolderMutationVariables = Exact<{
  name: Scalars['String'];
}>;


export type CreateFolderMutation = { __typename?: 'Mutation', createFolder: { __typename?: 'Folder', id: string, name: string } };

export type RenameFolderMutationVariables = Exact<{
  id: Scalars['ID'];
  name: Scalars['String'];
}>;


export type RenameFolderMutation = { __typename?: 'Mutation', renameFolder: { __typename?: 'Folder', id: string, name: string } };

export type DeleteFolderMutationVariables = Exact<{
  id: Scalars['ID'];
}>;


export type DeleteFolderMutation = { __typename?: 'Mutation', deleteFolder: { __typename?: 'Folder', id: string } };

export type MovePlaylistToFolderMutationVariables = Exact<{
  playlistId: Scalars['ID'];
  folderId: Scalars['ID'];
}>;


export type MovePlaylistToFolderMutation = { __typename?: 'Mutation', movePlaylistToFolder: { __typename?: 'Folder', id: string, name: string, playlists: Array<{ __typename?: 'Playlist', id: string, name: string, description?: string | null }> } };

export type MovePlaylistsToFolderMutationVariables = Exact<{
  playlistIds: Array<Scalars['ID']> | Scalars['ID'];
  folderId: Scalars['ID'];
}>;


export type MovePlaylistsToFolderMutation = { __typename?: 'Mutation', movePlaylistsToFolder: { __typename?: 'Folder', id: string, name: string, playlists: Array<{ __typename?: 'Playlist', id: string, name: string, description?: string | null }> } };

export type GetPlaylistsQueryVariables = Exact<{ [key: string]: never; }>;


export type GetPlaylistsQuery = { __typename?: 'Query', playlists: Array<{ __typename?: 'Playlist', id: string, name: string, description?: string | null }> };

export type GetRecentPlaylistsQueryVariables = Exact<{ [key: string]: never; }>;


export type GetRecentPlaylistsQuery = { __typename?: 'Query', recentPlaylists: Array<{ __typename?: 'Playlist', id: string, name: string, description?: string | null }> };

export type GetMainPlaylistsQueryVariables = Exact<{ [key: string]: never; }>;


export type GetMainPlaylistsQuery = { __typename?: 'Query', mainPlaylists: Array<{ __typename?: 'Playlist', id: string, name: string, description?: string | null }> };

export type GetPlaylistQueryVariables = Exact<{
  id: Scalars['ID'];
}>;


export type GetPlaylistQuery = { __typename?: 'Query', playlist: { __typename?: 'Playlist', id: string, name: string, description?: string | null, tracks: Array<{ __typename?: 'Track', id: string, title: string, albumTitle: string, artist: string, artistId: string, albumId: string, cover?: string | null, duration?: number | null }> } };

export type GetFoldersQueryVariables = Exact<{ [key: string]: never; }>;


export type GetFoldersQuery = { __typename?: 'Query', folders: Array<{ __typename?: 'Folder', id: string, name: string }> };

export type GetFolderQueryVariables = Exact<{
  id: Scalars['ID'];
}>;


export type GetFolderQuery = { __typename?: 'Query', folder: { __typename?: 'Folder', id: string, name: string, playlists: Array<{ __typename?: 'Playlist', id: string, name: string, description?: string | null }> } };

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

export type PlayPlaylistMutationVariables = Exact<{
  playlistId: Scalars['ID'];
  position?: InputMaybe<Scalars['Int']>;
  shuffle: Scalars['Boolean'];
}>;


export type PlayPlaylistMutation = { __typename?: 'Mutation', playPlaylist: boolean };

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
export const PlaylistFragmentFragmentDoc = gql`
    fragment PlaylistFragment on Playlist {
  id
  name
  description
  tracks {
    id
    title
    albumTitle
    artist
    artistId
    albumId
    cover
    duration
  }
}
    `;
export const FolderFragmentFragmentDoc = gql`
    fragment FolderFragment on Folder {
  id
  name
  playlists {
    id
    name
    description
  }
}
    `;
export const ListDevicesDocument = gql`
    query ListDevices {
  listDevices {
    id
    app
    name
    service
    host
    port
    isConnected
  }
}
    `;

/**
 * __useListDevicesQuery__
 *
 * To run a query within a React component, call `useListDevicesQuery` and pass it any options that fit your needs.
 * When your component renders, `useListDevicesQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useListDevicesQuery({
 *   variables: {
 *   },
 * });
 */
export function useListDevicesQuery(baseOptions?: Apollo.QueryHookOptions<ListDevicesQuery, ListDevicesQueryVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useQuery<ListDevicesQuery, ListDevicesQueryVariables>(ListDevicesDocument, options);
      }
export function useListDevicesLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<ListDevicesQuery, ListDevicesQueryVariables>) {
          const options = {...defaultOptions, ...baseOptions}
          return Apollo.useLazyQuery<ListDevicesQuery, ListDevicesQueryVariables>(ListDevicesDocument, options);
        }
export type ListDevicesQueryHookResult = ReturnType<typeof useListDevicesQuery>;
export type ListDevicesLazyQueryHookResult = ReturnType<typeof useListDevicesLazyQuery>;
export type ListDevicesQueryResult = Apollo.QueryResult<ListDevicesQuery, ListDevicesQueryVariables>;
export const OnNewDeviceDocument = gql`
    subscription OnNewDevice {
  onNewDevice {
    id
    app
    name
    service
    host
    port
    isConnected
  }
}
    `;

/**
 * __useOnNewDeviceSubscription__
 *
 * To run a query within a React component, call `useOnNewDeviceSubscription` and pass it any options that fit your needs.
 * When your component renders, `useOnNewDeviceSubscription` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the subscription, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useOnNewDeviceSubscription({
 *   variables: {
 *   },
 * });
 */
export function useOnNewDeviceSubscription(baseOptions?: Apollo.SubscriptionHookOptions<OnNewDeviceSubscription, OnNewDeviceSubscriptionVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useSubscription<OnNewDeviceSubscription, OnNewDeviceSubscriptionVariables>(OnNewDeviceDocument, options);
      }
export type OnNewDeviceSubscriptionHookResult = ReturnType<typeof useOnNewDeviceSubscription>;
export type OnNewDeviceSubscriptionResult = Apollo.SubscriptionResult<OnNewDeviceSubscription>;
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
export const SearchDocument = gql`
    query Search($keyword: String!) {
  search(keyword: $keyword) {
    artists {
      id
      name
      picture
    }
    albums {
      id
      title
      artist
      cover
    }
    tracks {
      id
      title
      artist
      duration
      cover
      artistId
      albumId
      albumTitle
    }
  }
}
    `;

/**
 * __useSearchQuery__
 *
 * To run a query within a React component, call `useSearchQuery` and pass it any options that fit your needs.
 * When your component renders, `useSearchQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useSearchQuery({
 *   variables: {
 *      keyword: // value for 'keyword'
 *   },
 * });
 */
export function useSearchQuery(baseOptions: Apollo.QueryHookOptions<SearchQuery, SearchQueryVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useQuery<SearchQuery, SearchQueryVariables>(SearchDocument, options);
      }
export function useSearchLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<SearchQuery, SearchQueryVariables>) {
          const options = {...defaultOptions, ...baseOptions}
          return Apollo.useLazyQuery<SearchQuery, SearchQueryVariables>(SearchDocument, options);
        }
export type SearchQueryHookResult = ReturnType<typeof useSearchQuery>;
export type SearchLazyQueryHookResult = ReturnType<typeof useSearchLazyQuery>;
export type SearchQueryResult = Apollo.QueryResult<SearchQuery, SearchQueryVariables>;
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
export const PlayerStateChangedDocument = gql`
    subscription PlayerStateChanged {
  playerState {
    isPlaying
  }
}
    `;

/**
 * __usePlayerStateChangedSubscription__
 *
 * To run a query within a React component, call `usePlayerStateChangedSubscription` and pass it any options that fit your needs.
 * When your component renders, `usePlayerStateChangedSubscription` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the subscription, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = usePlayerStateChangedSubscription({
 *   variables: {
 *   },
 * });
 */
export function usePlayerStateChangedSubscription(baseOptions?: Apollo.SubscriptionHookOptions<PlayerStateChangedSubscription, PlayerStateChangedSubscriptionVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useSubscription<PlayerStateChangedSubscription, PlayerStateChangedSubscriptionVariables>(PlayerStateChangedDocument, options);
      }
export type PlayerStateChangedSubscriptionHookResult = ReturnType<typeof usePlayerStateChangedSubscription>;
export type PlayerStateChangedSubscriptionResult = Apollo.SubscriptionResult<PlayerStateChangedSubscription>;
export const TrackTimePositionChangedDocument = gql`
    subscription TrackTimePositionChanged {
  trackTimePosition {
    positionMs
  }
}
    `;

/**
 * __useTrackTimePositionChangedSubscription__
 *
 * To run a query within a React component, call `useTrackTimePositionChangedSubscription` and pass it any options that fit your needs.
 * When your component renders, `useTrackTimePositionChangedSubscription` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the subscription, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useTrackTimePositionChangedSubscription({
 *   variables: {
 *   },
 * });
 */
export function useTrackTimePositionChangedSubscription(baseOptions?: Apollo.SubscriptionHookOptions<TrackTimePositionChangedSubscription, TrackTimePositionChangedSubscriptionVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useSubscription<TrackTimePositionChangedSubscription, TrackTimePositionChangedSubscriptionVariables>(TrackTimePositionChangedDocument, options);
      }
export type TrackTimePositionChangedSubscriptionHookResult = ReturnType<typeof useTrackTimePositionChangedSubscription>;
export type TrackTimePositionChangedSubscriptionResult = Apollo.SubscriptionResult<TrackTimePositionChangedSubscription>;
export const CurrentlyPlayingSongChangedDocument = gql`
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
    ${TrackFragmentFragmentDoc}`;

/**
 * __useCurrentlyPlayingSongChangedSubscription__
 *
 * To run a query within a React component, call `useCurrentlyPlayingSongChangedSubscription` and pass it any options that fit your needs.
 * When your component renders, `useCurrentlyPlayingSongChangedSubscription` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the subscription, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useCurrentlyPlayingSongChangedSubscription({
 *   variables: {
 *   },
 * });
 */
export function useCurrentlyPlayingSongChangedSubscription(baseOptions?: Apollo.SubscriptionHookOptions<CurrentlyPlayingSongChangedSubscription, CurrentlyPlayingSongChangedSubscriptionVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useSubscription<CurrentlyPlayingSongChangedSubscription, CurrentlyPlayingSongChangedSubscriptionVariables>(CurrentlyPlayingSongChangedDocument, options);
      }
export type CurrentlyPlayingSongChangedSubscriptionHookResult = ReturnType<typeof useCurrentlyPlayingSongChangedSubscription>;
export type CurrentlyPlayingSongChangedSubscriptionResult = Apollo.SubscriptionResult<CurrentlyPlayingSongChangedSubscription>;
export const CreatePlaylistDocument = gql`
    mutation CreatePlaylist($name: String!, $description: String) {
  createPlaylist(name: $name, description: $description) {
    id
    name
    description
  }
}
    `;
export type CreatePlaylistMutationFn = Apollo.MutationFunction<CreatePlaylistMutation, CreatePlaylistMutationVariables>;

/**
 * __useCreatePlaylistMutation__
 *
 * To run a mutation, you first call `useCreatePlaylistMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `useCreatePlaylistMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [createPlaylistMutation, { data, loading, error }] = useCreatePlaylistMutation({
 *   variables: {
 *      name: // value for 'name'
 *      description: // value for 'description'
 *   },
 * });
 */
export function useCreatePlaylistMutation(baseOptions?: Apollo.MutationHookOptions<CreatePlaylistMutation, CreatePlaylistMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<CreatePlaylistMutation, CreatePlaylistMutationVariables>(CreatePlaylistDocument, options);
      }
export type CreatePlaylistMutationHookResult = ReturnType<typeof useCreatePlaylistMutation>;
export type CreatePlaylistMutationResult = Apollo.MutationResult<CreatePlaylistMutation>;
export type CreatePlaylistMutationOptions = Apollo.BaseMutationOptions<CreatePlaylistMutation, CreatePlaylistMutationVariables>;
export const DeletePlaylistDocument = gql`
    mutation DeletePlaylist($id: ID!) {
  deletePlaylist(id: $id) {
    id
  }
}
    `;
export type DeletePlaylistMutationFn = Apollo.MutationFunction<DeletePlaylistMutation, DeletePlaylistMutationVariables>;

/**
 * __useDeletePlaylistMutation__
 *
 * To run a mutation, you first call `useDeletePlaylistMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `useDeletePlaylistMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [deletePlaylistMutation, { data, loading, error }] = useDeletePlaylistMutation({
 *   variables: {
 *      id: // value for 'id'
 *   },
 * });
 */
export function useDeletePlaylistMutation(baseOptions?: Apollo.MutationHookOptions<DeletePlaylistMutation, DeletePlaylistMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<DeletePlaylistMutation, DeletePlaylistMutationVariables>(DeletePlaylistDocument, options);
      }
export type DeletePlaylistMutationHookResult = ReturnType<typeof useDeletePlaylistMutation>;
export type DeletePlaylistMutationResult = Apollo.MutationResult<DeletePlaylistMutation>;
export type DeletePlaylistMutationOptions = Apollo.BaseMutationOptions<DeletePlaylistMutation, DeletePlaylistMutationVariables>;
export const AddTrackToPlaylistDocument = gql`
    mutation AddTrackToPlaylist($playlistId: ID!, $trackId: ID!) {
  addTrackToPlaylist(id: $playlistId, trackId: $trackId) {
    ...PlaylistFragment
  }
}
    ${PlaylistFragmentFragmentDoc}`;
export type AddTrackToPlaylistMutationFn = Apollo.MutationFunction<AddTrackToPlaylistMutation, AddTrackToPlaylistMutationVariables>;

/**
 * __useAddTrackToPlaylistMutation__
 *
 * To run a mutation, you first call `useAddTrackToPlaylistMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `useAddTrackToPlaylistMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [addTrackToPlaylistMutation, { data, loading, error }] = useAddTrackToPlaylistMutation({
 *   variables: {
 *      playlistId: // value for 'playlistId'
 *      trackId: // value for 'trackId'
 *   },
 * });
 */
export function useAddTrackToPlaylistMutation(baseOptions?: Apollo.MutationHookOptions<AddTrackToPlaylistMutation, AddTrackToPlaylistMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<AddTrackToPlaylistMutation, AddTrackToPlaylistMutationVariables>(AddTrackToPlaylistDocument, options);
      }
export type AddTrackToPlaylistMutationHookResult = ReturnType<typeof useAddTrackToPlaylistMutation>;
export type AddTrackToPlaylistMutationResult = Apollo.MutationResult<AddTrackToPlaylistMutation>;
export type AddTrackToPlaylistMutationOptions = Apollo.BaseMutationOptions<AddTrackToPlaylistMutation, AddTrackToPlaylistMutationVariables>;
export const RemoveTrackFromPlaylistDocument = gql`
    mutation RemoveTrackFromPlaylist($playlistId: ID!, $position: Int!) {
  removeTrackFromPlaylist(id: $playlistId, position: $position) {
    ...PlaylistFragment
  }
}
    ${PlaylistFragmentFragmentDoc}`;
export type RemoveTrackFromPlaylistMutationFn = Apollo.MutationFunction<RemoveTrackFromPlaylistMutation, RemoveTrackFromPlaylistMutationVariables>;

/**
 * __useRemoveTrackFromPlaylistMutation__
 *
 * To run a mutation, you first call `useRemoveTrackFromPlaylistMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `useRemoveTrackFromPlaylistMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [removeTrackFromPlaylistMutation, { data, loading, error }] = useRemoveTrackFromPlaylistMutation({
 *   variables: {
 *      playlistId: // value for 'playlistId'
 *      position: // value for 'position'
 *   },
 * });
 */
export function useRemoveTrackFromPlaylistMutation(baseOptions?: Apollo.MutationHookOptions<RemoveTrackFromPlaylistMutation, RemoveTrackFromPlaylistMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<RemoveTrackFromPlaylistMutation, RemoveTrackFromPlaylistMutationVariables>(RemoveTrackFromPlaylistDocument, options);
      }
export type RemoveTrackFromPlaylistMutationHookResult = ReturnType<typeof useRemoveTrackFromPlaylistMutation>;
export type RemoveTrackFromPlaylistMutationResult = Apollo.MutationResult<RemoveTrackFromPlaylistMutation>;
export type RemoveTrackFromPlaylistMutationOptions = Apollo.BaseMutationOptions<RemoveTrackFromPlaylistMutation, RemoveTrackFromPlaylistMutationVariables>;
export const RenamePlaylistDocument = gql`
    mutation RenamePlaylist($id: ID!, $name: String!) {
  renamePlaylist(id: $id, name: $name) {
    id
    name
  }
}
    `;
export type RenamePlaylistMutationFn = Apollo.MutationFunction<RenamePlaylistMutation, RenamePlaylistMutationVariables>;

/**
 * __useRenamePlaylistMutation__
 *
 * To run a mutation, you first call `useRenamePlaylistMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `useRenamePlaylistMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [renamePlaylistMutation, { data, loading, error }] = useRenamePlaylistMutation({
 *   variables: {
 *      id: // value for 'id'
 *      name: // value for 'name'
 *   },
 * });
 */
export function useRenamePlaylistMutation(baseOptions?: Apollo.MutationHookOptions<RenamePlaylistMutation, RenamePlaylistMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<RenamePlaylistMutation, RenamePlaylistMutationVariables>(RenamePlaylistDocument, options);
      }
export type RenamePlaylistMutationHookResult = ReturnType<typeof useRenamePlaylistMutation>;
export type RenamePlaylistMutationResult = Apollo.MutationResult<RenamePlaylistMutation>;
export type RenamePlaylistMutationOptions = Apollo.BaseMutationOptions<RenamePlaylistMutation, RenamePlaylistMutationVariables>;
export const CreateFolderDocument = gql`
    mutation CreateFolder($name: String!) {
  createFolder(name: $name) {
    id
    name
  }
}
    `;
export type CreateFolderMutationFn = Apollo.MutationFunction<CreateFolderMutation, CreateFolderMutationVariables>;

/**
 * __useCreateFolderMutation__
 *
 * To run a mutation, you first call `useCreateFolderMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `useCreateFolderMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [createFolderMutation, { data, loading, error }] = useCreateFolderMutation({
 *   variables: {
 *      name: // value for 'name'
 *   },
 * });
 */
export function useCreateFolderMutation(baseOptions?: Apollo.MutationHookOptions<CreateFolderMutation, CreateFolderMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<CreateFolderMutation, CreateFolderMutationVariables>(CreateFolderDocument, options);
      }
export type CreateFolderMutationHookResult = ReturnType<typeof useCreateFolderMutation>;
export type CreateFolderMutationResult = Apollo.MutationResult<CreateFolderMutation>;
export type CreateFolderMutationOptions = Apollo.BaseMutationOptions<CreateFolderMutation, CreateFolderMutationVariables>;
export const RenameFolderDocument = gql`
    mutation RenameFolder($id: ID!, $name: String!) {
  renameFolder(id: $id, name: $name) {
    id
    name
  }
}
    `;
export type RenameFolderMutationFn = Apollo.MutationFunction<RenameFolderMutation, RenameFolderMutationVariables>;

/**
 * __useRenameFolderMutation__
 *
 * To run a mutation, you first call `useRenameFolderMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `useRenameFolderMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [renameFolderMutation, { data, loading, error }] = useRenameFolderMutation({
 *   variables: {
 *      id: // value for 'id'
 *      name: // value for 'name'
 *   },
 * });
 */
export function useRenameFolderMutation(baseOptions?: Apollo.MutationHookOptions<RenameFolderMutation, RenameFolderMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<RenameFolderMutation, RenameFolderMutationVariables>(RenameFolderDocument, options);
      }
export type RenameFolderMutationHookResult = ReturnType<typeof useRenameFolderMutation>;
export type RenameFolderMutationResult = Apollo.MutationResult<RenameFolderMutation>;
export type RenameFolderMutationOptions = Apollo.BaseMutationOptions<RenameFolderMutation, RenameFolderMutationVariables>;
export const DeleteFolderDocument = gql`
    mutation DeleteFolder($id: ID!) {
  deleteFolder(id: $id) {
    id
  }
}
    `;
export type DeleteFolderMutationFn = Apollo.MutationFunction<DeleteFolderMutation, DeleteFolderMutationVariables>;

/**
 * __useDeleteFolderMutation__
 *
 * To run a mutation, you first call `useDeleteFolderMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `useDeleteFolderMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [deleteFolderMutation, { data, loading, error }] = useDeleteFolderMutation({
 *   variables: {
 *      id: // value for 'id'
 *   },
 * });
 */
export function useDeleteFolderMutation(baseOptions?: Apollo.MutationHookOptions<DeleteFolderMutation, DeleteFolderMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<DeleteFolderMutation, DeleteFolderMutationVariables>(DeleteFolderDocument, options);
      }
export type DeleteFolderMutationHookResult = ReturnType<typeof useDeleteFolderMutation>;
export type DeleteFolderMutationResult = Apollo.MutationResult<DeleteFolderMutation>;
export type DeleteFolderMutationOptions = Apollo.BaseMutationOptions<DeleteFolderMutation, DeleteFolderMutationVariables>;
export const MovePlaylistToFolderDocument = gql`
    mutation MovePlaylistToFolder($playlistId: ID!, $folderId: ID!) {
  movePlaylistToFolder(id: $playlistId, folderId: $folderId) {
    ...FolderFragment
  }
}
    ${FolderFragmentFragmentDoc}`;
export type MovePlaylistToFolderMutationFn = Apollo.MutationFunction<MovePlaylistToFolderMutation, MovePlaylistToFolderMutationVariables>;

/**
 * __useMovePlaylistToFolderMutation__
 *
 * To run a mutation, you first call `useMovePlaylistToFolderMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `useMovePlaylistToFolderMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [movePlaylistToFolderMutation, { data, loading, error }] = useMovePlaylistToFolderMutation({
 *   variables: {
 *      playlistId: // value for 'playlistId'
 *      folderId: // value for 'folderId'
 *   },
 * });
 */
export function useMovePlaylistToFolderMutation(baseOptions?: Apollo.MutationHookOptions<MovePlaylistToFolderMutation, MovePlaylistToFolderMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<MovePlaylistToFolderMutation, MovePlaylistToFolderMutationVariables>(MovePlaylistToFolderDocument, options);
      }
export type MovePlaylistToFolderMutationHookResult = ReturnType<typeof useMovePlaylistToFolderMutation>;
export type MovePlaylistToFolderMutationResult = Apollo.MutationResult<MovePlaylistToFolderMutation>;
export type MovePlaylistToFolderMutationOptions = Apollo.BaseMutationOptions<MovePlaylistToFolderMutation, MovePlaylistToFolderMutationVariables>;
export const MovePlaylistsToFolderDocument = gql`
    mutation MovePlaylistsToFolder($playlistIds: [ID!]!, $folderId: ID!) {
  movePlaylistsToFolder(ids: $playlistIds, folderId: $folderId) {
    ...FolderFragment
  }
}
    ${FolderFragmentFragmentDoc}`;
export type MovePlaylistsToFolderMutationFn = Apollo.MutationFunction<MovePlaylistsToFolderMutation, MovePlaylistsToFolderMutationVariables>;

/**
 * __useMovePlaylistsToFolderMutation__
 *
 * To run a mutation, you first call `useMovePlaylistsToFolderMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `useMovePlaylistsToFolderMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [movePlaylistsToFolderMutation, { data, loading, error }] = useMovePlaylistsToFolderMutation({
 *   variables: {
 *      playlistIds: // value for 'playlistIds'
 *      folderId: // value for 'folderId'
 *   },
 * });
 */
export function useMovePlaylistsToFolderMutation(baseOptions?: Apollo.MutationHookOptions<MovePlaylistsToFolderMutation, MovePlaylistsToFolderMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<MovePlaylistsToFolderMutation, MovePlaylistsToFolderMutationVariables>(MovePlaylistsToFolderDocument, options);
      }
export type MovePlaylistsToFolderMutationHookResult = ReturnType<typeof useMovePlaylistsToFolderMutation>;
export type MovePlaylistsToFolderMutationResult = Apollo.MutationResult<MovePlaylistsToFolderMutation>;
export type MovePlaylistsToFolderMutationOptions = Apollo.BaseMutationOptions<MovePlaylistsToFolderMutation, MovePlaylistsToFolderMutationVariables>;
export const GetPlaylistsDocument = gql`
    query GetPlaylists {
  playlists {
    id
    name
    description
  }
}
    `;

/**
 * __useGetPlaylistsQuery__
 *
 * To run a query within a React component, call `useGetPlaylistsQuery` and pass it any options that fit your needs.
 * When your component renders, `useGetPlaylistsQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useGetPlaylistsQuery({
 *   variables: {
 *   },
 * });
 */
export function useGetPlaylistsQuery(baseOptions?: Apollo.QueryHookOptions<GetPlaylistsQuery, GetPlaylistsQueryVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useQuery<GetPlaylistsQuery, GetPlaylistsQueryVariables>(GetPlaylistsDocument, options);
      }
export function useGetPlaylistsLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<GetPlaylistsQuery, GetPlaylistsQueryVariables>) {
          const options = {...defaultOptions, ...baseOptions}
          return Apollo.useLazyQuery<GetPlaylistsQuery, GetPlaylistsQueryVariables>(GetPlaylistsDocument, options);
        }
export type GetPlaylistsQueryHookResult = ReturnType<typeof useGetPlaylistsQuery>;
export type GetPlaylistsLazyQueryHookResult = ReturnType<typeof useGetPlaylistsLazyQuery>;
export type GetPlaylistsQueryResult = Apollo.QueryResult<GetPlaylistsQuery, GetPlaylistsQueryVariables>;
export const GetRecentPlaylistsDocument = gql`
    query GetRecentPlaylists {
  recentPlaylists {
    id
    name
    description
  }
}
    `;

/**
 * __useGetRecentPlaylistsQuery__
 *
 * To run a query within a React component, call `useGetRecentPlaylistsQuery` and pass it any options that fit your needs.
 * When your component renders, `useGetRecentPlaylistsQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useGetRecentPlaylistsQuery({
 *   variables: {
 *   },
 * });
 */
export function useGetRecentPlaylistsQuery(baseOptions?: Apollo.QueryHookOptions<GetRecentPlaylistsQuery, GetRecentPlaylistsQueryVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useQuery<GetRecentPlaylistsQuery, GetRecentPlaylistsQueryVariables>(GetRecentPlaylistsDocument, options);
      }
export function useGetRecentPlaylistsLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<GetRecentPlaylistsQuery, GetRecentPlaylistsQueryVariables>) {
          const options = {...defaultOptions, ...baseOptions}
          return Apollo.useLazyQuery<GetRecentPlaylistsQuery, GetRecentPlaylistsQueryVariables>(GetRecentPlaylistsDocument, options);
        }
export type GetRecentPlaylistsQueryHookResult = ReturnType<typeof useGetRecentPlaylistsQuery>;
export type GetRecentPlaylistsLazyQueryHookResult = ReturnType<typeof useGetRecentPlaylistsLazyQuery>;
export type GetRecentPlaylistsQueryResult = Apollo.QueryResult<GetRecentPlaylistsQuery, GetRecentPlaylistsQueryVariables>;
export const GetMainPlaylistsDocument = gql`
    query GetMainPlaylists {
  mainPlaylists {
    id
    name
    description
  }
}
    `;

/**
 * __useGetMainPlaylistsQuery__
 *
 * To run a query within a React component, call `useGetMainPlaylistsQuery` and pass it any options that fit your needs.
 * When your component renders, `useGetMainPlaylistsQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useGetMainPlaylistsQuery({
 *   variables: {
 *   },
 * });
 */
export function useGetMainPlaylistsQuery(baseOptions?: Apollo.QueryHookOptions<GetMainPlaylistsQuery, GetMainPlaylistsQueryVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useQuery<GetMainPlaylistsQuery, GetMainPlaylistsQueryVariables>(GetMainPlaylistsDocument, options);
      }
export function useGetMainPlaylistsLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<GetMainPlaylistsQuery, GetMainPlaylistsQueryVariables>) {
          const options = {...defaultOptions, ...baseOptions}
          return Apollo.useLazyQuery<GetMainPlaylistsQuery, GetMainPlaylistsQueryVariables>(GetMainPlaylistsDocument, options);
        }
export type GetMainPlaylistsQueryHookResult = ReturnType<typeof useGetMainPlaylistsQuery>;
export type GetMainPlaylistsLazyQueryHookResult = ReturnType<typeof useGetMainPlaylistsLazyQuery>;
export type GetMainPlaylistsQueryResult = Apollo.QueryResult<GetMainPlaylistsQuery, GetMainPlaylistsQueryVariables>;
export const GetPlaylistDocument = gql`
    query GetPlaylist($id: ID!) {
  playlist(id: $id) {
    ...PlaylistFragment
  }
}
    ${PlaylistFragmentFragmentDoc}`;

/**
 * __useGetPlaylistQuery__
 *
 * To run a query within a React component, call `useGetPlaylistQuery` and pass it any options that fit your needs.
 * When your component renders, `useGetPlaylistQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useGetPlaylistQuery({
 *   variables: {
 *      id: // value for 'id'
 *   },
 * });
 */
export function useGetPlaylistQuery(baseOptions: Apollo.QueryHookOptions<GetPlaylistQuery, GetPlaylistQueryVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useQuery<GetPlaylistQuery, GetPlaylistQueryVariables>(GetPlaylistDocument, options);
      }
export function useGetPlaylistLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<GetPlaylistQuery, GetPlaylistQueryVariables>) {
          const options = {...defaultOptions, ...baseOptions}
          return Apollo.useLazyQuery<GetPlaylistQuery, GetPlaylistQueryVariables>(GetPlaylistDocument, options);
        }
export type GetPlaylistQueryHookResult = ReturnType<typeof useGetPlaylistQuery>;
export type GetPlaylistLazyQueryHookResult = ReturnType<typeof useGetPlaylistLazyQuery>;
export type GetPlaylistQueryResult = Apollo.QueryResult<GetPlaylistQuery, GetPlaylistQueryVariables>;
export const GetFoldersDocument = gql`
    query GetFolders {
  folders {
    id
    name
  }
}
    `;

/**
 * __useGetFoldersQuery__
 *
 * To run a query within a React component, call `useGetFoldersQuery` and pass it any options that fit your needs.
 * When your component renders, `useGetFoldersQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useGetFoldersQuery({
 *   variables: {
 *   },
 * });
 */
export function useGetFoldersQuery(baseOptions?: Apollo.QueryHookOptions<GetFoldersQuery, GetFoldersQueryVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useQuery<GetFoldersQuery, GetFoldersQueryVariables>(GetFoldersDocument, options);
      }
export function useGetFoldersLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<GetFoldersQuery, GetFoldersQueryVariables>) {
          const options = {...defaultOptions, ...baseOptions}
          return Apollo.useLazyQuery<GetFoldersQuery, GetFoldersQueryVariables>(GetFoldersDocument, options);
        }
export type GetFoldersQueryHookResult = ReturnType<typeof useGetFoldersQuery>;
export type GetFoldersLazyQueryHookResult = ReturnType<typeof useGetFoldersLazyQuery>;
export type GetFoldersQueryResult = Apollo.QueryResult<GetFoldersQuery, GetFoldersQueryVariables>;
export const GetFolderDocument = gql`
    query GetFolder($id: ID!) {
  folder(id: $id) {
    ...FolderFragment
  }
}
    ${FolderFragmentFragmentDoc}`;

/**
 * __useGetFolderQuery__
 *
 * To run a query within a React component, call `useGetFolderQuery` and pass it any options that fit your needs.
 * When your component renders, `useGetFolderQuery` returns an object from Apollo Client that contains loading, error, and data properties
 * you can use to render your UI.
 *
 * @param baseOptions options that will be passed into the query, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options;
 *
 * @example
 * const { data, loading, error } = useGetFolderQuery({
 *   variables: {
 *      id: // value for 'id'
 *   },
 * });
 */
export function useGetFolderQuery(baseOptions: Apollo.QueryHookOptions<GetFolderQuery, GetFolderQueryVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useQuery<GetFolderQuery, GetFolderQueryVariables>(GetFolderDocument, options);
      }
export function useGetFolderLazyQuery(baseOptions?: Apollo.LazyQueryHookOptions<GetFolderQuery, GetFolderQueryVariables>) {
          const options = {...defaultOptions, ...baseOptions}
          return Apollo.useLazyQuery<GetFolderQuery, GetFolderQueryVariables>(GetFolderDocument, options);
        }
export type GetFolderQueryHookResult = ReturnType<typeof useGetFolderQuery>;
export type GetFolderLazyQueryHookResult = ReturnType<typeof useGetFolderLazyQuery>;
export type GetFolderQueryResult = Apollo.QueryResult<GetFolderQuery, GetFolderQueryVariables>;
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
export const PlayPlaylistDocument = gql`
    mutation PlayPlaylist($playlistId: ID!, $position: Int, $shuffle: Boolean!) {
  playPlaylist(id: $playlistId, position: $position, shuffle: $shuffle)
}
    `;
export type PlayPlaylistMutationFn = Apollo.MutationFunction<PlayPlaylistMutation, PlayPlaylistMutationVariables>;

/**
 * __usePlayPlaylistMutation__
 *
 * To run a mutation, you first call `usePlayPlaylistMutation` within a React component and pass it any options that fit your needs.
 * When your component renders, `usePlayPlaylistMutation` returns a tuple that includes:
 * - A mutate function that you can call at any time to execute the mutation
 * - An object with fields that represent the current status of the mutation's execution
 *
 * @param baseOptions options that will be passed into the mutation, supported options are listed on: https://www.apollographql.com/docs/react/api/react-hooks/#options-2;
 *
 * @example
 * const [playPlaylistMutation, { data, loading, error }] = usePlayPlaylistMutation({
 *   variables: {
 *      playlistId: // value for 'playlistId'
 *      position: // value for 'position'
 *      shuffle: // value for 'shuffle'
 *   },
 * });
 */
export function usePlayPlaylistMutation(baseOptions?: Apollo.MutationHookOptions<PlayPlaylistMutation, PlayPlaylistMutationVariables>) {
        const options = {...defaultOptions, ...baseOptions}
        return Apollo.useMutation<PlayPlaylistMutation, PlayPlaylistMutationVariables>(PlayPlaylistDocument, options);
      }
export type PlayPlaylistMutationHookResult = ReturnType<typeof usePlayPlaylistMutation>;
export type PlayPlaylistMutationResult = Apollo.MutationResult<PlayPlaylistMutation>;
export type PlayPlaylistMutationOptions = Apollo.BaseMutationOptions<PlayPlaylistMutation, PlayPlaylistMutationVariables>;
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