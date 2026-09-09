import { useMutation, useQuery, useInfiniteQuery, UseMutationOptions, UseQueryOptions, UseInfiniteQueryOptions, InfiniteData } from '@tanstack/react-query';
import { fetcher } from '../Api/fetcher';
export type Maybe<T> = T | null;
export type InputMaybe<T> = Maybe<T>;
export type Exact<T extends { [key: string]: unknown }> = { [K in keyof T]: T[K] };
export type MakeOptional<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]?: Maybe<T[SubKey]> };
export type MakeMaybe<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]: Maybe<T[SubKey]> };
export type MakeEmpty<T extends { [key: string]: unknown }, K extends keyof T> = { [_ in K]?: never };
export type Incremental<T> = T | { [P in keyof T]?: P extends ' $fragmentName' | '__typename' ? T[P] : never };
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: { input: string; output: string; }
  String: { input: string; output: string; }
  Boolean: { input: boolean; output: boolean; }
  Int: { input: number; output: number; }
  Float: { input: number; output: number; }
};

export type Album = {
  __typename?: 'Album';
  artist: Scalars['String']['output'];
  cover?: Maybe<Scalars['String']['output']>;
  genres: Array<Scalars['String']['output']>;
  id: Scalars['String']['output'];
  releaseDate: Scalars['String']['output'];
  /** Which library this row came from; `null` is this machine. */
  source?: Maybe<Scalars['String']['output']>;
  title: Scalars['String']['output'];
  tracks: Array<Track>;
  year?: Maybe<Scalars['Int']['output']>;
};

export enum App {
  Jellyfin = 'JELLYFIN',
  MusicPlayer = 'MUSIC_PLAYER',
  Subsonic = 'SUBSONIC'
}

export type Artist = {
  __typename?: 'Artist';
  albums: Array<Album>;
  bio: Scalars['String']['output'];
  genres: Array<Scalars['String']['output']>;
  id: Scalars['String']['output'];
  images: Array<Scalars['String']['output']>;
  name: Scalars['String']['output'];
  picture: Scalars['String']['output'];
  songs: Array<Track>;
  /** Which library this row came from; `null` is this machine. */
  source?: Maybe<Scalars['String']['output']>;
  website: Scalars['String']['output'];
};

/**
 * The DSP chain's state, in the firmware's units.
 *
 * Mirrors the `GetAudioSettingsResponse` of the gRPC `MixerService`, field for
 * field, so the web client and the Slint desktop show the same numbers. The
 * ranges come back with the values because they are what a slider needs and
 * they are not obvious from the field alone.
 */
export type AudioSettingsState = {
  __typename?: 'AudioSettingsState';
  /** Stereo balance, -100 (full left)..=100 (full right). */
  balance: Scalars['Int']['output'];
  /** Bass shelf gain in dB. */
  bass: Scalars['Int']['output'];
  bassMax: Scalars['Int']['output'];
  bassMin: Scalars['Int']['output'];
  /** 0 off … 5 always. */
  crossfade: Scalars['Int']['output'];
  dithering: Scalars['Boolean']['output'];
  eqBands: Array<EqBand>;
  eqEnabled: Scalars['Boolean']['output'];
  /** EQ pre-gain in dB × 10, 0..=240. */
  eqPrecut: Scalars['Int']['output'];
  fadeInDelay: Scalars['Int']['output'];
  fadeInDuration: Scalars['Int']['output'];
  fadeOutDelay: Scalars['Int']['output'];
  fadeOutDuration: Scalars['Int']['output'];
  /** 0 crossfade, 2 mix. */
  fadeOutMixmode: Scalars['Int']['output'];
  replaygainNoclip: Scalars['Boolean']['output'];
  /** ReplayGain pre-amp in dB × 10, -120..=120. */
  replaygainPreamp: Scalars['Int']['output'];
  /** 0 track, 1 album, 2 track (shuffle), 3 off. */
  replaygainType: Scalars['Int']['output'];
  /** Treble shelf gain in dB. */
  treble: Scalars['Int']['output'];
  trebleMax: Scalars['Int']['output'];
  trebleMin: Scalars['Int']['output'];
};

export type ConnectedDevice = {
  __typename?: 'ConnectedDevice';
  app: Scalars['String']['output'];
  host: Scalars['String']['output'];
  id: Scalars['String']['output'];
  isConnected: Scalars['Boolean']['output'];
  name: Scalars['String']['output'];
  port: Scalars['Int']['output'];
  service: Scalars['String']['output'];
};

export type CurrentlyPlayingSong = {
  __typename?: 'CurrentlyPlayingSong';
  index: Scalars['Int']['output'];
  isPlaying: Scalars['Boolean']['output'];
  positionMs: Scalars['Int']['output'];
  track?: Maybe<Track>;
};

export type Device = {
  __typename?: 'Device';
  app: Scalars['String']['output'];
  host: Scalars['String']['output'];
  id: Scalars['String']['output'];
  isConnected: Scalars['Boolean']['output'];
  name: Scalars['String']['output'];
  port: Scalars['Int']['output'];
  service: Scalars['String']['output'];
};

export type DisconnectedDevice = {
  __typename?: 'DisconnectedDevice';
  app: Scalars['String']['output'];
  host: Scalars['String']['output'];
  id: Scalars['String']['output'];
  isConnected: Scalars['Boolean']['output'];
  name: Scalars['String']['output'];
  port: Scalars['Int']['output'];
  service: Scalars['String']['output'];
};

/** One EQ band, in the firmware's units. */
export type EqBand = {
  __typename?: 'EqBand';
  /** Centre frequency in Hz. */
  cutoff: Scalars['Int']['output'];
  /**
   * Gain in dB, times ten — the engine's own unit, so a UI never has to
   * round-trip a float through a slider.
   */
  gain: Scalars['Int']['output'];
  /** Q, times ten. */
  q: Scalars['Int']['output'];
};

/** An installed WebAssembly extension, as its manifest declares it. */
export type Extension = {
  __typename?: 'Extension';
  /** Hosts it may reach over HTTP. Empty means no network at all. */
  allowedHosts: Array<Scalars['String']['output']>;
  author: Scalars['String']['output'];
  /**
   * What it plugs into: `events`, `metadata`, `commands`, `predicates`,
   * `source`. An extension only ever receives calls for what it declared.
   */
  capabilities: Array<Scalars['String']['output']>;
  description: Scalars['String']['output'];
  homepage: Scalars['String']['output'];
  /** Reverse-DNS id, e.g. `com.example.lyrics`. Stable across versions. */
  id: Scalars['ID']['output'];
  /** Whether it may query the library through the host functions. */
  libraryRead: Scalars['Boolean']['output'];
  license: Scalars['String']['output'];
  /** A URL, or a file name relative to the extension's directory. */
  logo: Scalars['String']['output'];
  name: Scalars['String']['output'];
  /** The directory it was read from, so a user can find it on disk. */
  path: Scalars['String']['output'];
  repository: Scalars['String']['output'];
  /**
   * `enabled` or `disabled`.
   *
   * This is a manifest scan, so it cannot report whether an enabled module
   * actually loads — that needs the module. The daemon logs a load failure
   * at startup.
   */
  status: Scalars['String']['output'];
  /** Free-form tags, e.g. `["lyrics", "offline"]`. */
  topics: Array<Scalars['String']['output']>;
  version: Scalars['String']['output'];
};

export type Folder = {
  __typename?: 'Folder';
  id: Scalars['String']['output'];
  name: Scalars['String']['output'];
  playlists: Array<Playlist>;
};

export type FolderChanged = {
  __typename?: 'FolderChanged';
  folder: Folder;
  mutationType: MutationType;
  playlist?: Maybe<Playlist>;
};

export type Genre = {
  __typename?: 'Genre';
  id: Scalars['ID']['output'];
  name: Scalars['String']['output'];
  trackCount: Scalars['Int']['output'];
};

/** Output levels for a meter, measured on the PCM leaving the device. */
export type Levels = {
  __typename?: 'Levels';
  /** 0..1 RMS over one output buffer. */
  left: Scalars['Float']['output'];
  /**
   * The same signal below roughly 200 Hz, which is what makes a meter move
   * with the bass rather than with whatever is loudest.
   */
  lowLeft: Scalars['Float']['output'];
  lowRight: Scalars['Float']['output'];
  right: Scalars['Float']['output'];
};

export type Mutation = {
  __typename?: 'Mutation';
  /**
   * Add a station the user typed in themselves, after checking the url is
   * actually a reachable stream. Signed in to atradio.fm, it is published as
   * a `fm.atradio.station` record so it follows the account everywhere.
   */
  addRadioStation: RadioStation;
  /** Save a server, or update the one already stored at that url. */
  addServer: Server;
  addTrack: Array<Track>;
  addTrackToPlaylist: Playlist;
  addTracks: Scalars['Boolean']['output'];
  clearTracklist: Scalars['Boolean']['output'];
  connectToCastDevice: Device;
  /**
   * Read the library from a *discovered* device — an mDNS peer, or a
   * streaming server named in the settings.
   *
   * A saved server goes through `connectToServer` instead. Both end at the
   * same `ProviderState::connect`, which is what stops the two paths from
   * disagreeing about what "connected" means.
   */
  connectToDevice: Device;
  /**
   * Point the library screens at a saved server.
   *
   * Connects first and swaps second, so a server that is unreachable leaves
   * the previous one in place. Playback is untouched either way.
   */
  connectToServer: Server;
  createFolder: Folder;
  /**
   * Create a playlist.
   *
   * Passing `smart` makes it a smart playlist: the filter is validated and
   * the tracks generated immediately, so the caller gets back a playlist
   * that is already populated rather than an empty one to fill by hand.
   */
  createPlaylist: Playlist;
  deleteFolder: Folder;
  deletePlaylist: Playlist;
  /**
   * Forget a server. Disconnects first if it is the one in use, so the
   * screens fall back to the local library rather than reading from
   * something that is no longer listed.
   */
  deleteServer: Scalars['Boolean']['output'];
  disconnectFromCastDevice?: Maybe<Device>;
  disconnectFromDevice?: Maybe<Device>;
  /** Back to the local library. Returns what was disconnected. */
  disconnectFromServer?: Maybe<Server>;
  /**
   * Like or unlike a track.
   *
   * With a provider connected the like belongs to *that* server, so it goes
   * there. A provider with no notion of likes says so rather than quietly
   * writing an id to Rocksky that means nothing there.
   */
  likeTrack: Scalars['Boolean']['output'];
  movePlaylistToFolder: Folder;
  movePlaylistsToFolder: Folder;
  next: Scalars['Boolean']['output'];
  pause: Scalars['Boolean']['output'];
  play: Scalars['Boolean']['output'];
  playAlbum: Scalars['Boolean']['output'];
  playArtistTracks: Scalars['Boolean']['output'];
  playNext: Scalars['Boolean']['output'];
  playPlaylist: Scalars['Boolean']['output'];
  playRadio: Scalars['Boolean']['output'];
  playTrackAt: Scalars['Boolean']['output'];
  previous: Scalars['Boolean']['output'];
  /** Re-run a smart playlist's filter against the library as it is now. */
  regenerateSmartPlaylist: Playlist;
  removeSavedRadio: Scalars['Boolean']['output'];
  removeTrack: Scalars['Boolean']['output'];
  removeTrackFromPlaylist: Playlist;
  renameFolder: Folder;
  renamePlaylist: Playlist;
  /**
   * Re-read the extension directories and return what is installed now.
   *
   * Also drops the stored flags for extensions that are no longer there: a
   * leftover row is harmless while it matches nothing, but it would switch
   * an extension straight back off if it were ever reinstalled, which is
   * not what deleting it meant.
   */
  rescanExtensions: Array<Extension>;
  saveRadio: Scalars['Boolean']['output'];
  /**
   * Always the local music directory: a scan is about the files on this
   * machine, whatever the screens happen to be pointed at.
   */
  scan: Scalars['Boolean']['output'];
  seek: Scalars['Boolean']['output'];
  /**
   * Change one audio setting, by the same names the gRPC service takes —
   * `eq_enabled`, `eq_precut`, `bass`, `treble`, `balance`, `rg_type`,
   * `rg_preamp`, `rg_noclip`, `crossfade`, `fade_in_delay`,
   * `fade_in_duration`, `fade_out_delay`, `fade_out_duration`,
   * `fade_out_mixmode`, `dithering`.
   *
   * Everything is an integer in the firmware's own units (dB × 10 where a
   * fraction is meaningful, 0/1 for a flag), which is what keeps a slider
   * from having to round-trip a float.
   *
   * Returns the whole state, so a client renders what was stored rather
   * than what it asked for — the two differ wherever a value was clamped.
   */
  setAudioSetting: AudioSettingsState;
  /** Set one EQ band's gain, in dB × 10 (-240..=240). */
  setEqBandGain: AudioSettingsState;
  /**
   * Switch an extension on or off, and return it as it now stands.
   *
   * The flag is stored immediately, but a module that is already loaded
   * keeps running until the daemon next starts — unloading WebAssembly
   * mid-session would pull the ground out from under whatever happens to be
   * in the middle of a call into it.
   */
  setExtensionEnabled: Extension;
  setMute: Scalars['Boolean']['output'];
  setVolume: Scalars['Boolean']['output'];
  shuffle: Scalars['Boolean']['output'];
  stop: Scalars['Boolean']['output'];
  /**
   * Bookmark or unbookmark whatever station is playing, and report the new
   * state — what the heart in the miniplayer needs.
   *
   * The station is rebuilt from the queued track rather than taken from the
   * caller, so the miniplayer can toggle a bookmark without having to know
   * where the station came from (it may be playing from a previous session).
   */
  toggleCurrentRadioBookmark: Scalars['Boolean']['output'];
  /** Change a smart playlist's filter and regenerate its tracks. */
  updateSmartPlaylist: Playlist;
};


export type MutationAddRadioStationArgs = {
  station: NewRadioStationInput;
};


export type MutationAddServerArgs = {
  input: ServerInput;
};


export type MutationAddTrackArgs = {
  track: TrackInput;
};


export type MutationAddTrackToPlaylistArgs = {
  id: Scalars['ID']['input'];
  trackId: Scalars['ID']['input'];
};


export type MutationAddTracksArgs = {
  tracks: Array<TrackInput>;
};


export type MutationConnectToCastDeviceArgs = {
  id: Scalars['ID']['input'];
};


export type MutationConnectToDeviceArgs = {
  id: Scalars['ID']['input'];
};


export type MutationConnectToServerArgs = {
  id: Scalars['ID']['input'];
};


export type MutationCreateFolderArgs = {
  name: Scalars['String']['input'];
};


export type MutationCreatePlaylistArgs = {
  description?: InputMaybe<Scalars['String']['input']>;
  folderId?: InputMaybe<Scalars['ID']['input']>;
  name: Scalars['String']['input'];
  smart?: InputMaybe<SmartPlaylistInput>;
};


export type MutationDeleteFolderArgs = {
  id: Scalars['ID']['input'];
};


export type MutationDeletePlaylistArgs = {
  id: Scalars['ID']['input'];
};


export type MutationDeleteServerArgs = {
  id: Scalars['ID']['input'];
};


export type MutationLikeTrackArgs = {
  id: Scalars['String']['input'];
  like: Scalars['Boolean']['input'];
};


export type MutationMovePlaylistToFolderArgs = {
  folderId: Scalars['ID']['input'];
  id: Scalars['ID']['input'];
};


export type MutationMovePlaylistsToFolderArgs = {
  folderId: Scalars['ID']['input'];
  ids: Array<Scalars['ID']['input']>;
};


export type MutationPlayAlbumArgs = {
  id: Scalars['ID']['input'];
  position?: InputMaybe<Scalars['Int']['input']>;
  shuffle: Scalars['Boolean']['input'];
};


export type MutationPlayArtistTracksArgs = {
  id: Scalars['ID']['input'];
  position?: InputMaybe<Scalars['Int']['input']>;
  shuffle: Scalars['Boolean']['input'];
};


export type MutationPlayNextArgs = {
  id: Scalars['ID']['input'];
};


export type MutationPlayPlaylistArgs = {
  id: Scalars['ID']['input'];
  position?: InputMaybe<Scalars['Int']['input']>;
  shuffle: Scalars['Boolean']['input'];
};


export type MutationPlayRadioArgs = {
  station: RadioStationInput;
};


export type MutationPlayTrackAtArgs = {
  position: Scalars['Int']['input'];
};


export type MutationRegenerateSmartPlaylistArgs = {
  id: Scalars['ID']['input'];
};


export type MutationRemoveSavedRadioArgs = {
  id: Scalars['String']['input'];
};


export type MutationRemoveTrackArgs = {
  position: Scalars['Int']['input'];
};


export type MutationRemoveTrackFromPlaylistArgs = {
  id: Scalars['ID']['input'];
  position: Scalars['Int']['input'];
};


export type MutationRenameFolderArgs = {
  id: Scalars['ID']['input'];
  name: Scalars['String']['input'];
};


export type MutationRenamePlaylistArgs = {
  id: Scalars['ID']['input'];
  name: Scalars['String']['input'];
};


export type MutationRescanExtensionsArgs = {
  filter?: InputMaybe<Scalars['String']['input']>;
};


export type MutationSaveRadioArgs = {
  station: RadioStationInput;
};


export type MutationSeekArgs = {
  position: Scalars['Int']['input'];
};


export type MutationSetAudioSettingArgs = {
  name: Scalars['String']['input'];
  value: Scalars['Int']['input'];
};


export type MutationSetEqBandGainArgs = {
  band: Scalars['Int']['input'];
  gain: Scalars['Int']['input'];
};


export type MutationSetExtensionEnabledArgs = {
  enabled: Scalars['Boolean']['input'];
  id: Scalars['ID']['input'];
};


export type MutationSetMuteArgs = {
  mute: Scalars['Boolean']['input'];
};


export type MutationSetVolumeArgs = {
  volume: Scalars['Int']['input'];
};


export type MutationUpdateSmartPlaylistArgs = {
  id: Scalars['ID']['input'];
  smart: SmartPlaylistInput;
};

export enum MutationType {
  Cleared = 'CLEARED',
  Created = 'CREATED',
  Deleted = 'DELETED',
  Moved = 'MOVED',
  Renamed = 'RENAMED',
  Updated = 'UPDATED'
}

/**
 * The fields the "add station" form collects. The id is derived from the
 * stream url (or handed out by atradio once published), so it is not asked for.
 */
export type NewRadioStationInput = {
  country?: Scalars['String']['input'];
  genre?: Scalars['String']['input'];
  logo?: Scalars['String']['input'];
  name: Scalars['String']['input'];
  streamUrl: Scalars['String']['input'];
};

export type PlayerState = {
  __typename?: 'PlayerState';
  index: Scalars['Int']['output'];
  isPlaying: Scalars['Boolean']['output'];
  positionMs: Scalars['Int']['output'];
};

export type Playlist = {
  __typename?: 'Playlist';
  description?: Maybe<Scalars['String']['output']>;
  id: Scalars['String']['output'];
  isSmart: Scalars['Boolean']['output'];
  maxTracks?: Maybe<Scalars['Int']['output']>;
  name: Scalars['String']['output'];
  /** The RSQL filter behind a smart playlist, e.g. `genre==rock;year>2000`. */
  rsql?: Maybe<Scalars['String']['output']>;
  sortBy?: Maybe<Scalars['String']['output']>;
  sortOrder?: Maybe<Scalars['String']['output']>;
  /**
   * Never zero for a playlist that has tracks, whether or not this
   * response carried them.
   */
  trackCount: Scalars['Int']['output'];
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
  positionMs: Scalars['Int']['output'];
};

export type Query = {
  __typename?: 'Query';
  album: Album;
  albums: Array<Album>;
  artist: Artist;
  artists: Array<Artist>;
  /** The DSP chain's state: EQ, tone, ReplayGain, crossfade, dithering. */
  audioSettings: AudioSettingsState;
  /** Check a stream url before the "add station" form saves it. */
  checkRadioStream: StreamCheck;
  connectedCastDevice: Device;
  /** The device the library is being read from, if it is a discovered one. */
  connectedDevice: Device;
  /**
   * The server the library screens are currently reading from, if any.
   * `null` means the local library.
   */
  connectedServer?: Maybe<Server>;
  currentlyPlayingSong: CurrentlyPlayingSong;
  /**
   * Every installed extension, in id order. `filter` is a case-insensitive
   * search over the id, name, description, author, topics and capabilities.
   *
   * This reads the manifests rather than loading the modules, so opening an
   * extensions page costs a directory walk instead of one WebAssembly
   * instantiation per extension.
   */
  extensions: Array<Extension>;
  folder: Folder;
  folders: Array<Folder>;
  /** The tracks in one genre. */
  genreTracks: Array<Track>;
  /**
   * The genres in the library.
   *
   * From the connected server when it has them, else this machine's own
   * table — a server that cannot list genres should show the local ones
   * rather than an empty screen.
   */
  genres: Array<Genre>;
  getMute: Scalars['Boolean']['output'];
  getNextTrack?: Maybe<Track>;
  getPlayerState: PlayerState;
  getPreviousTrack?: Maybe<Track>;
  getRandom: Scalars['Boolean']['output'];
  getRepeat: Scalars['Boolean']['output'];
  getVolume: Scalars['Int']['output'];
  /**
   * The tracks the user has liked.
   *
   * On a remote provider these are *its* likes — Subsonic's starred songs,
   * Jellyfin's favourites. It must not fall back to the local list: those
   * ids belong to a different library, so the rows would render but not
   * play. A provider with no notion of likes returns nothing, which is an
   * empty screen rather than a wrong one.
   */
  likedTracks: Array<Track>;
  listCastDevices: Array<Device>;
  listDevices: Array<Device>;
  mainPlaylists: Array<Playlist>;
  playlist: Playlist;
  playlists: Array<Playlist>;
  /**
   * The user's own stations — the ones added by hand here, on atradio.fm, or
   * on another device. A subset of the bookmarks, told apart by their source.
   */
  radioStations: Array<RadioStation>;
  radios: Array<RadioStation>;
  recentPlaylists: Array<Playlist>;
  /**
   * Every field a smart-playlist filter may mention, so a form can offer a
   * picker rather than expecting the vocabulary to be memorised.
   */
  rsqlFields: Array<RsqlField>;
  savedRadios: Array<RadioStation>;
  savedServer?: Maybe<Server>;
  /** Every saved server, with the connected one flagged. */
  savedServers: Array<Server>;
  /**
   * Search wherever the library currently is.
   *
   * The local searcher indexes local files, so with a provider connected it
   * would be answering about a library the user is not looking at.
   * Search both libraries at once.
   *
   * With a provider connected the results are *federated*: the remote
   * server and this machine's own index are queried together and the rows
   * interleaved, so one search box covers everything reachable rather than
   * silently describing only whichever library happens to be current.
   *
   * The two run concurrently — a remote round trip should not be paid on
   * top of a local index scan — and a failure on either side yields that
   * side's results as empty rather than failing the whole search: half an
   * answer is worth more than none.
   */
  search: SearchResult;
  /**
   * What a smart-playlist filter would produce, without saving it.
   *
   * Cheap enough to call as the user types: it selects ids, then loads only
   * the handful it shows. A filter that does not compile comes back as an
   * error carrying the offset of the offending character.
   */
  smartPlaylistPreview: SmartPlaylistPreview;
  /** The kinds of server this build can talk to, straight from the registry. */
  sourceKinds: Array<SourceKind>;
  track: Track;
  tracklistTracks: Tracklist;
  tracks: Array<Track>;
};


export type QueryAlbumArgs = {
  id: Scalars['ID']['input'];
};


export type QueryAlbumsArgs = {
  filter?: InputMaybe<Scalars['String']['input']>;
  limit?: InputMaybe<Scalars['Int']['input']>;
  offset?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryArtistArgs = {
  id: Scalars['ID']['input'];
};


export type QueryArtistsArgs = {
  filter?: InputMaybe<Scalars['String']['input']>;
  limit?: InputMaybe<Scalars['Int']['input']>;
  offset?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryCheckRadioStreamArgs = {
  url: Scalars['String']['input'];
};


export type QueryExtensionsArgs = {
  filter?: InputMaybe<Scalars['String']['input']>;
};


export type QueryFolderArgs = {
  id: Scalars['ID']['input'];
};


export type QueryGenreTracksArgs = {
  id: Scalars['ID']['input'];
  limit?: InputMaybe<Scalars['Int']['input']>;
  offset?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryGenresArgs = {
  limit?: InputMaybe<Scalars['Int']['input']>;
  offset?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryLikedTracksArgs = {
  limit?: InputMaybe<Scalars['Int']['input']>;
  offset?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryListDevicesArgs = {
  filter?: InputMaybe<App>;
};


export type QueryPlaylistArgs = {
  id: Scalars['ID']['input'];
};


export type QueryPlaylistsArgs = {
  limit?: InputMaybe<Scalars['Int']['input']>;
  offset?: InputMaybe<Scalars['Int']['input']>;
};


export type QueryRadiosArgs = {
  category?: InputMaybe<Scalars['String']['input']>;
  query?: InputMaybe<Scalars['String']['input']>;
};


export type QuerySavedServerArgs = {
  id: Scalars['ID']['input'];
};


export type QuerySearchArgs = {
  keyword: Scalars['String']['input'];
};


export type QuerySmartPlaylistPreviewArgs = {
  sample?: Scalars['Int']['input'];
  smart: SmartPlaylistInput;
};


export type QueryTrackArgs = {
  id: Scalars['ID']['input'];
};


export type QueryTracksArgs = {
  filter?: InputMaybe<Scalars['String']['input']>;
  limit?: InputMaybe<Scalars['Int']['input']>;
  offset?: InputMaybe<Scalars['Int']['input']>;
};

export type RadioStation = {
  __typename?: 'RadioStation';
  bitrate: Scalars['Int']['output'];
  country: Scalars['String']['output'];
  genre: Scalars['String']['output'];
  id: Scalars['String']['output'];
  logo: Scalars['String']['output'];
  name: Scalars['String']['output'];
  source: Scalars['String']['output'];
  streamUrl: Scalars['String']['output'];
};

export type RadioStationInput = {
  bitrate: Scalars['Int']['input'];
  country: Scalars['String']['input'];
  genre: Scalars['String']['input'];
  id: Scalars['String']['input'];
  logo: Scalars['String']['input'];
  name: Scalars['String']['input'];
  source: Scalars['String']['input'];
  streamUrl: Scalars['String']['input'];
};

/**
 * One filterable field, so a UI can offer a picker instead of making the user
 * remember the vocabulary.
 */
export type RsqlField = {
  __typename?: 'RsqlField';
  /** `text`, `integer`, `boolean` or `timestamp` — what values it accepts. */
  kind: Scalars['String']['output'];
  /** A human label. */
  label: Scalars['String']['output'];
  /** The name to write in a filter. */
  name: Scalars['String']['output'];
};

export type SearchResult = {
  __typename?: 'SearchResult';
  albums: Array<Album>;
  artists: Array<Artist>;
  tracks: Array<Track>;
};

/**
 * A remote server the user has saved.
 *
 * Note the absence of `password`. It is stored, and it is sent when a server
 * is added or edited, but it is never returned — so an edit form has to
 * re-prompt for it rather than round-tripping a secret through a client.
 */
export type Server = {
  __typename?: 'Server';
  /**
   * Whether this is the server the library screens are currently reading
   * from.
   */
  connected: Scalars['Boolean']['output'];
  /** Whether a password is stored, since the password itself is not exposed. */
  hasPassword: Scalars['Boolean']['output'];
  id: Scalars['ID']['output'];
  /** The provider-registry key: `subsonic`, `jellyfin`, `music-player`, … */
  kind: Scalars['String']['output'];
  name: Scalars['String']['output'];
  url: Scalars['String']['output'];
  username?: Maybe<Scalars['String']['output']>;
};

export type ServerInput = {
  kind: Scalars['String']['input'];
  name: Scalars['String']['input'];
  /**
   * Omit to keep whatever is already stored — the edit form never receives
   * the current one, so a blank field cannot mean "clear it".
   */
  password?: InputMaybe<Scalars['String']['input']>;
  url: Scalars['String']['input'];
  username?: InputMaybe<Scalars['String']['input']>;
};

/** The three things a smart playlist is: a filter, an order, and a cap. */
export type SmartPlaylistInput = {
  /**
   * RSQL over the track fields, e.g. `genre==rock;year>2000`. Empty matches
   * the whole library.
   */
  filter?: Scalars['String']['input'];
  /** Maximum number of tracks. Absent or 0 is unlimited. */
  limit?: InputMaybe<Scalars['Int']['input']>;
  /** A track field to order by, or `random`. */
  sortBy?: InputMaybe<Scalars['String']['input']>;
  /** `asc` or `desc`. */
  sortOrder?: InputMaybe<Scalars['String']['input']>;
};

/**
 * What a filter would produce, without saving anything — the "142 tracks
 * match" line under the filter box, plus the first few by name.
 */
export type SmartPlaylistPreview = {
  __typename?: 'SmartPlaylistPreview';
  /** How many tracks the filter matches. */
  count: Scalars['Int']['output'];
  /** The first handful, so the form can show what it caught. */
  tracks: Array<Track>;
};

/**
 * What a client needs to offer one kind of server in its add-server form.
 *
 * Comes straight from the provider registry, so a newly registered backend
 * appears in every client without either of them being changed.
 */
export type SourceKind = {
  __typename?: 'SourceKind';
  defaultPort: Scalars['Int']['output'];
  displayName: Scalars['String']['output'];
  /**
   * Set for a backend that always talks to one address, so the form can
   * drop the url field instead of asking for something it ignores.
   */
  fixedUrl?: Maybe<Scalars['String']['output']>;
  kind: Scalars['String']['output'];
  /**
   * False for backends with no login — the form hides those fields rather
   * than asking for something that will be ignored.
   */
  needsCredentials: Scalars['Boolean']['output'];
};

/**
 * The verdict on a stream url, so the form can say what is wrong before the
 * station is saved — and fill itself in from the station's own ICY headers
 * when it is right.
 */
export type StreamCheck = {
  __typename?: 'StreamCheck';
  bitrate: Scalars['Int']['output'];
  codec: Scalars['String']['output'];
  error: Scalars['String']['output'];
  genre: Scalars['String']['output'];
  homepage: Scalars['String']['output'];
  name: Scalars['String']['output'];
  ok: Scalars['Boolean']['output'];
};

export type Subscription = {
  __typename?: 'Subscription';
  currentlyPlayingSong: Track;
  folder: FolderChanged;
  folders: Array<Folder>;
  /**
   * Output levels for a meter, pushed at 20 Hz.
   *
   * A stream rather than a field: a meter wants tens of updates a second,
   * and the poll a client would otherwise ride on is far slower. Read from
   * the tracklist, which the player writes each tick, so this adds no work
   * to the audio path.
   */
  levels: Levels;
  onConnected: ConnectedDevice;
  onDisconnected: DisconnectedDevice;
  onNewDevice: Device;
  playerState: PlayerState;
  playlist: PlaylistChanged;
  playlists: Array<Playlist>;
  trackTimePosition: PositionMilliseconds;
  tracklist: TracklistChanged;
};


export type SubscriptionFolderArgs = {
  id: Scalars['ID']['input'];
};


export type SubscriptionPlaylistArgs = {
  id: Scalars['ID']['input'];
};


export type SubscriptionTracklistArgs = {
  id: Scalars['ID']['input'];
};

export type Track = {
  __typename?: 'Track';
  album: Album;
  albumId: Scalars['String']['output'];
  albumTitle: Scalars['String']['output'];
  artist: Scalars['String']['output'];
  artistId: Scalars['String']['output'];
  artists: Array<Artist>;
  cover?: Maybe<Scalars['String']['output']>;
  discNumber: Scalars['Int']['output'];
  duration?: Maybe<Scalars['Float']['output']>;
  id: Scalars['String']['output'];
  /** Which library this row came from; `null` is this machine. */
  source?: Maybe<Scalars['String']['output']>;
  title: Scalars['String']['output'];
  trackNumber?: Maybe<Scalars['Int']['output']>;
  uri: Scalars['String']['output'];
};

export type TrackInput = {
  discNumber: Scalars['Int']['input'];
  duration?: InputMaybe<Scalars['Float']['input']>;
  id: Scalars['ID']['input'];
  title: Scalars['String']['input'];
  trackNumber?: InputMaybe<Scalars['Int']['input']>;
  uri: Scalars['String']['input'];
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

export type ConnectToDeviceMutationVariables = Exact<{
  id: Scalars['ID']['input'];
}>;


export type ConnectToDeviceMutation = { __typename?: 'Mutation', connectToDevice: { __typename?: 'Device', id: string } };

export type DisconnectFromDeviceMutationVariables = Exact<{ [key: string]: never; }>;


export type DisconnectFromDeviceMutation = { __typename?: 'Mutation', disconnectFromDevice?: { __typename?: 'Device', id: string } | null };

export type ConnectToCastDeviceMutationVariables = Exact<{
  id: Scalars['ID']['input'];
}>;


export type ConnectToCastDeviceMutation = { __typename?: 'Mutation', connectToCastDevice: { __typename?: 'Device', id: string } };

export type DisconnectFromCastDeviceMutationVariables = Exact<{ [key: string]: never; }>;


export type DisconnectFromCastDeviceMutation = { __typename?: 'Mutation', disconnectFromCastDevice?: { __typename?: 'Device', id: string } | null };

export type ListDevicesQueryVariables = Exact<{ [key: string]: never; }>;


export type ListDevicesQuery = { __typename?: 'Query', listDevices: Array<{ __typename?: 'Device', id: string, app: string, name: string, service: string, host: string, port: number, isConnected: boolean }> };

export type ListCastDevicesQueryVariables = Exact<{ [key: string]: never; }>;


export type ListCastDevicesQuery = { __typename?: 'Query', listCastDevices: Array<{ __typename?: 'Device', id: string, app: string, name: string, service: string, host: string, port: number, isConnected: boolean }> };

export type ConnectedDeviceQueryVariables = Exact<{ [key: string]: never; }>;


export type ConnectedDeviceQuery = { __typename?: 'Query', connectedDevice: { __typename?: 'Device', id: string, name: string, app: string, host: string, port: number, isConnected: boolean } };

export type ConnectedCastDeviceQueryVariables = Exact<{ [key: string]: never; }>;


export type ConnectedCastDeviceQuery = { __typename?: 'Query', connectedCastDevice: { __typename?: 'Device', id: string, name: string, app: string, host: string, port: number, isConnected: boolean } };

export type OnNewDeviceSubscriptionVariables = Exact<{ [key: string]: never; }>;


export type OnNewDeviceSubscription = { __typename?: 'Subscription', onNewDevice: { __typename?: 'Device', id: string, app: string, name: string, service: string, host: string, port: number, isConnected: boolean } };

export type OnDeviceConnectedSubscriptionVariables = Exact<{ [key: string]: never; }>;


export type OnDeviceConnectedSubscription = { __typename?: 'Subscription', onConnected: { __typename?: 'ConnectedDevice', id: string, name: string, service: string, app: string } };

export type OnDeviceDisconnectedSubscriptionVariables = Exact<{ [key: string]: never; }>;


export type OnDeviceDisconnectedSubscription = { __typename?: 'Subscription', onDisconnected: { __typename?: 'DisconnectedDevice', id: string, name: string, service: string, app: string } };

export type SetExtensionEnabledMutationVariables = Exact<{
  id: Scalars['ID']['input'];
  enabled: Scalars['Boolean']['input'];
}>;


export type SetExtensionEnabledMutation = { __typename?: 'Mutation', setExtensionEnabled: { __typename?: 'Extension', id: string, name: string, version: string, author: string, description: string, homepage: string, repository: string, license: string, logo: string, topics: Array<string>, capabilities: Array<string>, allowedHosts: Array<string>, libraryRead: boolean, status: string, path: string } };

export type RescanExtensionsMutationVariables = Exact<{
  filter?: InputMaybe<Scalars['String']['input']>;
}>;


export type RescanExtensionsMutation = { __typename?: 'Mutation', rescanExtensions: Array<{ __typename?: 'Extension', id: string, name: string, version: string, author: string, description: string, homepage: string, repository: string, license: string, logo: string, topics: Array<string>, capabilities: Array<string>, allowedHosts: Array<string>, libraryRead: boolean, status: string, path: string }> };

export type GetExtensionsQueryVariables = Exact<{
  filter?: InputMaybe<Scalars['String']['input']>;
}>;


export type GetExtensionsQuery = { __typename?: 'Query', extensions: Array<{ __typename?: 'Extension', id: string, name: string, version: string, author: string, description: string, homepage: string, repository: string, license: string, logo: string, topics: Array<string>, capabilities: Array<string>, allowedHosts: Array<string>, libraryRead: boolean, status: string, path: string }> };

export type AlbumFragmentFragment = { __typename?: 'Album', id: string, title: string, artist: string, year?: number | null, cover?: string | null };

export type ArtistFragmentFragment = { __typename?: 'Artist', id: string, name: string, picture: string };

export type TrackFragmentFragment = { __typename?: 'Track', id: string, trackNumber?: number | null, title: string, artist: string, duration?: number | null };

export type PlaylistFragmentFragment = { __typename?: 'Playlist', id: string, name: string, description?: string | null, tracks: Array<{ __typename?: 'Track', id: string, title: string, albumTitle: string, artist: string, artistId: string, albumId: string, cover?: string | null, duration?: number | null }> };

export type FolderFragmentFragment = { __typename?: 'Folder', id: string, name: string, playlists: Array<{ __typename?: 'Playlist', id: string, name: string, description?: string | null }> };

export type LikeTrackMutationVariables = Exact<{
  id: Scalars['String']['input'];
  like: Scalars['Boolean']['input'];
}>;


export type LikeTrackMutation = { __typename?: 'Mutation', likeTrack: boolean };

export type GetAlbumsQueryVariables = Exact<{
  filter?: InputMaybe<Scalars['String']['input']>;
  offset?: InputMaybe<Scalars['Int']['input']>;
  limit?: InputMaybe<Scalars['Int']['input']>;
}>;


export type GetAlbumsQuery = { __typename?: 'Query', albums: Array<{ __typename?: 'Album', id: string, title: string, artist: string, year?: number | null, cover?: string | null, tracks: Array<{ __typename?: 'Track', id: string, trackNumber?: number | null, title: string, artist: string, duration?: number | null }> }> };

export type GetArtistsQueryVariables = Exact<{
  filter?: InputMaybe<Scalars['String']['input']>;
  offset?: InputMaybe<Scalars['Int']['input']>;
  limit?: InputMaybe<Scalars['Int']['input']>;
}>;


export type GetArtistsQuery = { __typename?: 'Query', artists: Array<{ __typename?: 'Artist', id: string, name: string, picture: string }> };

export type GetTracksQueryVariables = Exact<{
  filter?: InputMaybe<Scalars['String']['input']>;
  offset?: InputMaybe<Scalars['Int']['input']>;
  limit?: InputMaybe<Scalars['Int']['input']>;
}>;


export type GetTracksQuery = { __typename?: 'Query', tracks: Array<{ __typename?: 'Track', id: string, trackNumber?: number | null, title: string, artist: string, duration?: number | null, artists: Array<{ __typename?: 'Artist', id: string, name: string }>, album: { __typename?: 'Album', id: string, title: string, artist: string, year?: number | null, cover?: string | null } }> };

export type GetArtistQueryVariables = Exact<{
  id: Scalars['ID']['input'];
}>;


export type GetArtistQuery = { __typename?: 'Query', artist: { __typename?: 'Artist', id: string, name: string, picture: string, songs: Array<{ __typename?: 'Track', id: string, title: string, artist: string, duration?: number | null, artists: Array<{ __typename?: 'Artist', id: string, name: string }>, album: { __typename?: 'Album', id: string, title: string, cover?: string | null } }>, albums: Array<{ __typename?: 'Album', id: string, title: string, artist: string, year?: number | null, cover?: string | null }> } };

export type GetAlbumQueryVariables = Exact<{
  id: Scalars['ID']['input'];
}>;


export type GetAlbumQuery = { __typename?: 'Query', album: { __typename?: 'Album', id: string, title: string, artist: string, year?: number | null, cover?: string | null, tracks: Array<{ __typename?: 'Track', id: string, trackNumber?: number | null, discNumber: number, title: string, artist: string, duration?: number | null, uri: string, artists: Array<{ __typename?: 'Artist', id: string, name: string }> }> } };

export type SearchQueryVariables = Exact<{
  keyword: Scalars['String']['input'];
}>;


export type SearchQuery = { __typename?: 'Query', search: { __typename?: 'SearchResult', artists: Array<{ __typename?: 'Artist', id: string, name: string, picture: string, source?: string | null }>, albums: Array<{ __typename?: 'Album', id: string, title: string, artist: string, cover?: string | null, source?: string | null }>, tracks: Array<{ __typename?: 'Track', id: string, title: string, artist: string, duration?: number | null, cover?: string | null, artistId: string, albumId: string, albumTitle: string, source?: string | null }> } };

export type GetGenresQueryVariables = Exact<{
  offset?: InputMaybe<Scalars['Int']['input']>;
  limit?: InputMaybe<Scalars['Int']['input']>;
}>;


export type GetGenresQuery = { __typename?: 'Query', genres: Array<{ __typename?: 'Genre', id: string, name: string, trackCount: number }> };

export type GetGenreTracksQueryVariables = Exact<{
  id: Scalars['ID']['input'];
  offset?: InputMaybe<Scalars['Int']['input']>;
  limit?: InputMaybe<Scalars['Int']['input']>;
}>;


export type GetGenreTracksQuery = { __typename?: 'Query', genreTracks: Array<{ __typename?: 'Track', id: string, title: string, artist: string, duration?: number | null, cover?: string | null, artistId: string, albumId: string, albumTitle: string }> };

export type SetAudioSettingMutationVariables = Exact<{
  name: Scalars['String']['input'];
  value: Scalars['Int']['input'];
}>;


export type SetAudioSettingMutation = { __typename?: 'Mutation', setAudioSetting: { __typename?: 'AudioSettingsState', eqEnabled: boolean, eqPrecut: number, bass: number, bassMin: number, bassMax: number, treble: number, trebleMin: number, trebleMax: number, balance: number, replaygainType: number, replaygainPreamp: number, replaygainNoclip: boolean, crossfade: number, fadeInDelay: number, fadeInDuration: number, fadeOutDelay: number, fadeOutDuration: number, fadeOutMixmode: number, dithering: boolean, eqBands: Array<{ __typename?: 'EqBand', cutoff: number, q: number, gain: number }> } };

export type SetEqBandGainMutationVariables = Exact<{
  band: Scalars['Int']['input'];
  gain: Scalars['Int']['input'];
}>;


export type SetEqBandGainMutation = { __typename?: 'Mutation', setEqBandGain: { __typename?: 'AudioSettingsState', eqEnabled: boolean, eqPrecut: number, bass: number, bassMin: number, bassMax: number, treble: number, trebleMin: number, trebleMax: number, balance: number, replaygainType: number, replaygainPreamp: number, replaygainNoclip: boolean, crossfade: number, fadeInDelay: number, fadeInDuration: number, fadeOutDelay: number, fadeOutDuration: number, fadeOutMixmode: number, dithering: boolean, eqBands: Array<{ __typename?: 'EqBand', cutoff: number, q: number, gain: number }> } };

export type AudioSettingsFieldsFragment = { __typename?: 'AudioSettingsState', eqEnabled: boolean, eqPrecut: number, bass: number, bassMin: number, bassMax: number, treble: number, trebleMin: number, trebleMax: number, balance: number, replaygainType: number, replaygainPreamp: number, replaygainNoclip: boolean, crossfade: number, fadeInDelay: number, fadeInDuration: number, fadeOutDelay: number, fadeOutDuration: number, fadeOutMixmode: number, dithering: boolean, eqBands: Array<{ __typename?: 'EqBand', cutoff: number, q: number, gain: number }> };

export type GetAudioSettingsQueryVariables = Exact<{ [key: string]: never; }>;


export type GetAudioSettingsQuery = { __typename?: 'Query', audioSettings: { __typename?: 'AudioSettingsState', eqEnabled: boolean, eqPrecut: number, bass: number, bassMin: number, bassMax: number, treble: number, trebleMin: number, trebleMax: number, balance: number, replaygainType: number, replaygainPreamp: number, replaygainNoclip: boolean, crossfade: number, fadeInDelay: number, fadeInDuration: number, fadeOutDelay: number, fadeOutDuration: number, fadeOutMixmode: number, dithering: boolean, eqBands: Array<{ __typename?: 'EqBand', cutoff: number, q: number, gain: number }> } };

export type NextMutationVariables = Exact<{ [key: string]: never; }>;


export type NextMutation = { __typename?: 'Mutation', next: boolean };

export type PauseMutationVariables = Exact<{ [key: string]: never; }>;


export type PauseMutation = { __typename?: 'Mutation', pause: boolean };

export type PlayMutationVariables = Exact<{ [key: string]: never; }>;


export type PlayMutation = { __typename?: 'Mutation', play: boolean };

export type PreviousMutationVariables = Exact<{ [key: string]: never; }>;


export type PreviousMutation = { __typename?: 'Mutation', previous: boolean };

export type SeekMutationVariables = Exact<{
  position: Scalars['Int']['input'];
}>;


export type SeekMutation = { __typename?: 'Mutation', seek: boolean };

export type CurrentlyPlayingSongQueryVariables = Exact<{ [key: string]: never; }>;


export type CurrentlyPlayingSongQuery = { __typename?: 'Query', currentlyPlayingSong: { __typename?: 'CurrentlyPlayingSong', index: number, isPlaying: boolean, positionMs: number, track?: { __typename?: 'Track', id: string, trackNumber?: number | null, title: string, artist: string, duration?: number | null, artists: Array<{ __typename?: 'Artist', id: string, name: string }>, album: { __typename?: 'Album', id: string, title: string, cover?: string | null } } | null } };

export type PlayerStateChangedSubscriptionVariables = Exact<{ [key: string]: never; }>;


export type PlayerStateChangedSubscription = { __typename?: 'Subscription', playerState: { __typename?: 'PlayerState', isPlaying: boolean } };

export type TrackTimePositionChangedSubscriptionVariables = Exact<{ [key: string]: never; }>;


export type TrackTimePositionChangedSubscription = { __typename?: 'Subscription', trackTimePosition: { __typename?: 'PositionMilliseconds', positionMs: number } };

export type CurrentlyPlayingSongChangedSubscriptionVariables = Exact<{ [key: string]: never; }>;


export type CurrentlyPlayingSongChangedSubscription = { __typename?: 'Subscription', currentlyPlayingSong: { __typename?: 'Track', id: string, trackNumber?: number | null, title: string, artist: string, duration?: number | null, artists: Array<{ __typename?: 'Artist', id: string, name: string }>, album: { __typename?: 'Album', id: string, title: string, cover?: string | null } } };

export type OnLevelsSubscriptionVariables = Exact<{ [key: string]: never; }>;


export type OnLevelsSubscription = { __typename?: 'Subscription', levels: { __typename?: 'Levels', left: number, right: number, lowLeft: number, lowRight: number } };

export type CreatePlaylistMutationVariables = Exact<{
  name: Scalars['String']['input'];
  description?: InputMaybe<Scalars['String']['input']>;
  smart?: InputMaybe<SmartPlaylistInput>;
}>;


export type CreatePlaylistMutation = { __typename?: 'Mutation', createPlaylist: { __typename?: 'Playlist', id: string, name: string, description?: string | null, isSmart: boolean, rsql?: string | null } };

export type DeletePlaylistMutationVariables = Exact<{
  id: Scalars['ID']['input'];
}>;


export type DeletePlaylistMutation = { __typename?: 'Mutation', deletePlaylist: { __typename?: 'Playlist', id: string } };

export type AddTrackToPlaylistMutationVariables = Exact<{
  playlistId: Scalars['ID']['input'];
  trackId: Scalars['ID']['input'];
}>;


export type AddTrackToPlaylistMutation = { __typename?: 'Mutation', addTrackToPlaylist: { __typename?: 'Playlist', id: string, name: string, description?: string | null, tracks: Array<{ __typename?: 'Track', id: string, title: string, albumTitle: string, artist: string, artistId: string, albumId: string, cover?: string | null, duration?: number | null }> } };

export type RemoveTrackFromPlaylistMutationVariables = Exact<{
  playlistId: Scalars['ID']['input'];
  position: Scalars['Int']['input'];
}>;


export type RemoveTrackFromPlaylistMutation = { __typename?: 'Mutation', removeTrackFromPlaylist: { __typename?: 'Playlist', id: string, name: string, description?: string | null, tracks: Array<{ __typename?: 'Track', id: string, title: string, albumTitle: string, artist: string, artistId: string, albumId: string, cover?: string | null, duration?: number | null }> } };

export type RenamePlaylistMutationVariables = Exact<{
  id: Scalars['ID']['input'];
  name: Scalars['String']['input'];
}>;


export type RenamePlaylistMutation = { __typename?: 'Mutation', renamePlaylist: { __typename?: 'Playlist', id: string, name: string } };

export type CreateFolderMutationVariables = Exact<{
  name: Scalars['String']['input'];
}>;


export type CreateFolderMutation = { __typename?: 'Mutation', createFolder: { __typename?: 'Folder', id: string, name: string } };

export type RenameFolderMutationVariables = Exact<{
  id: Scalars['ID']['input'];
  name: Scalars['String']['input'];
}>;


export type RenameFolderMutation = { __typename?: 'Mutation', renameFolder: { __typename?: 'Folder', id: string, name: string } };

export type DeleteFolderMutationVariables = Exact<{
  id: Scalars['ID']['input'];
}>;


export type DeleteFolderMutation = { __typename?: 'Mutation', deleteFolder: { __typename?: 'Folder', id: string } };

export type MovePlaylistToFolderMutationVariables = Exact<{
  playlistId: Scalars['ID']['input'];
  folderId: Scalars['ID']['input'];
}>;


export type MovePlaylistToFolderMutation = { __typename?: 'Mutation', movePlaylistToFolder: { __typename?: 'Folder', id: string, name: string, playlists: Array<{ __typename?: 'Playlist', id: string, name: string, description?: string | null }> } };

export type MovePlaylistsToFolderMutationVariables = Exact<{
  playlistIds: Array<Scalars['ID']['input']> | Scalars['ID']['input'];
  folderId: Scalars['ID']['input'];
}>;


export type MovePlaylistsToFolderMutation = { __typename?: 'Mutation', movePlaylistsToFolder: { __typename?: 'Folder', id: string, name: string, playlists: Array<{ __typename?: 'Playlist', id: string, name: string, description?: string | null }> } };

export type GetPlaylistsQueryVariables = Exact<{ [key: string]: never; }>;


export type GetPlaylistsQuery = { __typename?: 'Query', playlists: Array<{ __typename?: 'Playlist', id: string, name: string, description?: string | null, trackCount: number }> };

export type GetRecentPlaylistsQueryVariables = Exact<{ [key: string]: never; }>;


export type GetRecentPlaylistsQuery = { __typename?: 'Query', recentPlaylists: Array<{ __typename?: 'Playlist', id: string, name: string, description?: string | null, trackCount: number }> };

export type GetMainPlaylistsQueryVariables = Exact<{ [key: string]: never; }>;


export type GetMainPlaylistsQuery = { __typename?: 'Query', mainPlaylists: Array<{ __typename?: 'Playlist', id: string, name: string, description?: string | null, trackCount: number }> };

export type GetPlaylistQueryVariables = Exact<{
  id: Scalars['ID']['input'];
}>;


export type GetPlaylistQuery = { __typename?: 'Query', playlist: { __typename?: 'Playlist', id: string, name: string, description?: string | null, tracks: Array<{ __typename?: 'Track', id: string, title: string, albumTitle: string, artist: string, artistId: string, albumId: string, cover?: string | null, duration?: number | null }> } };

export type GetFoldersQueryVariables = Exact<{ [key: string]: never; }>;


export type GetFoldersQuery = { __typename?: 'Query', folders: Array<{ __typename?: 'Folder', id: string, name: string }> };

export type GetFolderQueryVariables = Exact<{
  id: Scalars['ID']['input'];
}>;


export type GetFolderQuery = { __typename?: 'Query', folder: { __typename?: 'Folder', id: string, name: string, playlists: Array<{ __typename?: 'Playlist', id: string, name: string, description?: string | null }> } };

export type AddServerMutationVariables = Exact<{
  input: ServerInput;
}>;


export type AddServerMutation = { __typename?: 'Mutation', addServer: { __typename?: 'Server', id: string, kind: string, name: string, url: string, username?: string | null, hasPassword: boolean, connected: boolean } };

export type DeleteServerMutationVariables = Exact<{
  id: Scalars['ID']['input'];
}>;


export type DeleteServerMutation = { __typename?: 'Mutation', deleteServer: boolean };

export type ConnectToServerMutationVariables = Exact<{
  id: Scalars['ID']['input'];
}>;


export type ConnectToServerMutation = { __typename?: 'Mutation', connectToServer: { __typename?: 'Server', id: string, kind: string, name: string, url: string, username?: string | null, hasPassword: boolean, connected: boolean } };

export type DisconnectFromServerMutationVariables = Exact<{ [key: string]: never; }>;


export type DisconnectFromServerMutation = { __typename?: 'Mutation', disconnectFromServer?: { __typename?: 'Server', id: string, name: string } | null };

export type ServerFragmentFragment = { __typename?: 'Server', id: string, kind: string, name: string, url: string, username?: string | null, hasPassword: boolean, connected: boolean };

export type GetSavedServersQueryVariables = Exact<{ [key: string]: never; }>;


export type GetSavedServersQuery = { __typename?: 'Query', savedServers: Array<{ __typename?: 'Server', id: string, kind: string, name: string, url: string, username?: string | null, hasPassword: boolean, connected: boolean }> };

export type GetSourceKindsQueryVariables = Exact<{ [key: string]: never; }>;


export type GetSourceKindsQuery = { __typename?: 'Query', sourceKinds: Array<{ __typename?: 'SourceKind', kind: string, displayName: string, needsCredentials: boolean, defaultPort: number, fixedUrl?: string | null }> };

export type GetConnectedServerQueryVariables = Exact<{ [key: string]: never; }>;


export type GetConnectedServerQuery = { __typename?: 'Query', connectedServer?: { __typename?: 'Server', id: string, kind: string, name: string, url: string, username?: string | null, hasPassword: boolean, connected: boolean } | null };

export type ClearTracklistMutationVariables = Exact<{ [key: string]: never; }>;


export type ClearTracklistMutation = { __typename?: 'Mutation', clearTracklist: boolean };

export type PlayAlbumMutationVariables = Exact<{
  albumId: Scalars['ID']['input'];
  position?: InputMaybe<Scalars['Int']['input']>;
  shuffle: Scalars['Boolean']['input'];
}>;


export type PlayAlbumMutation = { __typename?: 'Mutation', playAlbum: boolean };

export type PlayArtistTracksMutationVariables = Exact<{
  artistId: Scalars['ID']['input'];
  position?: InputMaybe<Scalars['Int']['input']>;
  shuffle: Scalars['Boolean']['input'];
}>;


export type PlayArtistTracksMutation = { __typename?: 'Mutation', playArtistTracks: boolean };

export type PlayPlaylistMutationVariables = Exact<{
  playlistId: Scalars['ID']['input'];
  position?: InputMaybe<Scalars['Int']['input']>;
  shuffle: Scalars['Boolean']['input'];
}>;


export type PlayPlaylistMutation = { __typename?: 'Mutation', playPlaylist: boolean };

export type PlayTrackAtMutationVariables = Exact<{
  position: Scalars['Int']['input'];
}>;


export type PlayTrackAtMutation = { __typename?: 'Mutation', playTrackAt: boolean };

export type RemoveTrackAtMutationVariables = Exact<{
  position: Scalars['Int']['input'];
}>;


export type RemoveTrackAtMutation = { __typename?: 'Mutation', removeTrack: boolean };

export type PlayNextMutationVariables = Exact<{
  trackId: Scalars['ID']['input'];
}>;


export type PlayNextMutation = { __typename?: 'Mutation', playNext: boolean };

export type AddTracksMutationVariables = Exact<{
  tracks: Array<TrackInput> | TrackInput;
}>;


export type AddTracksMutation = { __typename?: 'Mutation', addTracks: boolean };

export type GetTracklistQueryVariables = Exact<{ [key: string]: never; }>;


export type GetTracklistQuery = { __typename?: 'Query', tracklistTracks: { __typename?: 'Tracklist', previousTracks: Array<{ __typename?: 'Track', id: string, title: string, duration?: number | null, artists: Array<{ __typename?: 'Artist', id: string, name: string }>, album: { __typename?: 'Album', id: string, title: string, cover?: string | null } }>, nextTracks: Array<{ __typename?: 'Track', id: string, title: string, duration?: number | null, artists: Array<{ __typename?: 'Artist', id: string, name: string }>, album: { __typename?: 'Album', id: string, title: string, cover?: string | null } }> }, currentlyPlayingSong: { __typename?: 'CurrentlyPlayingSong', index: number, isPlaying: boolean, positionMs: number, track?: { __typename?: 'Track', id: string, trackNumber?: number | null, title: string, artist: string, duration?: number | null, artists: Array<{ __typename?: 'Artist', name: string }>, album: { __typename?: 'Album', title: string } } | null } };


export const AlbumFragmentFragmentDoc = `
    fragment AlbumFragment on Album {
  id
  title
  artist
  year
  cover
}
    `;
export const ArtistFragmentFragmentDoc = `
    fragment ArtistFragment on Artist {
  id
  name
  picture
}
    `;
export const TrackFragmentFragmentDoc = `
    fragment TrackFragment on Track {
  id
  trackNumber
  title
  artist
  duration
}
    `;
export const PlaylistFragmentFragmentDoc = `
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
export const FolderFragmentFragmentDoc = `
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
export const AudioSettingsFieldsFragmentDoc = `
    fragment AudioSettingsFields on AudioSettingsState {
  eqEnabled
  eqPrecut
  eqBands {
    cutoff
    q
    gain
  }
  bass
  bassMin
  bassMax
  treble
  trebleMin
  trebleMax
  balance
  replaygainType
  replaygainPreamp
  replaygainNoclip
  crossfade
  fadeInDelay
  fadeInDuration
  fadeOutDelay
  fadeOutDuration
  fadeOutMixmode
  dithering
}
    `;
export const ServerFragmentFragmentDoc = `
    fragment ServerFragment on Server {
  id
  kind
  name
  url
  username
  hasPassword
  connected
}
    `;
export const ConnectToDeviceDocument = `
    mutation ConnectToDevice($id: ID!) {
  connectToDevice(id: $id) {
    id
  }
}
    `;

export const useConnectToDeviceMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<ConnectToDeviceMutation, TError, ConnectToDeviceMutationVariables, TContext>) => {
    
    return useMutation<ConnectToDeviceMutation, TError, ConnectToDeviceMutationVariables, TContext>(
      {
    mutationKey: ['ConnectToDevice'],
    mutationFn: (variables?: ConnectToDeviceMutationVariables) => fetcher<ConnectToDeviceMutation, ConnectToDeviceMutationVariables>(ConnectToDeviceDocument, variables)(),
    ...options
  }
    )};

useConnectToDeviceMutation.getKey = () => ['ConnectToDevice'];


useConnectToDeviceMutation.fetcher = (variables: ConnectToDeviceMutationVariables, options?: RequestInit['headers']) => fetcher<ConnectToDeviceMutation, ConnectToDeviceMutationVariables>(ConnectToDeviceDocument, variables, options);

export const DisconnectFromDeviceDocument = `
    mutation DisconnectFromDevice {
  disconnectFromDevice {
    id
  }
}
    `;

export const useDisconnectFromDeviceMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<DisconnectFromDeviceMutation, TError, DisconnectFromDeviceMutationVariables, TContext>) => {
    
    return useMutation<DisconnectFromDeviceMutation, TError, DisconnectFromDeviceMutationVariables, TContext>(
      {
    mutationKey: ['DisconnectFromDevice'],
    mutationFn: (variables?: DisconnectFromDeviceMutationVariables) => fetcher<DisconnectFromDeviceMutation, DisconnectFromDeviceMutationVariables>(DisconnectFromDeviceDocument, variables)(),
    ...options
  }
    )};

useDisconnectFromDeviceMutation.getKey = () => ['DisconnectFromDevice'];


useDisconnectFromDeviceMutation.fetcher = (variables?: DisconnectFromDeviceMutationVariables, options?: RequestInit['headers']) => fetcher<DisconnectFromDeviceMutation, DisconnectFromDeviceMutationVariables>(DisconnectFromDeviceDocument, variables, options);

export const ConnectToCastDeviceDocument = `
    mutation ConnectToCastDevice($id: ID!) {
  connectToCastDevice(id: $id) {
    id
  }
}
    `;

export const useConnectToCastDeviceMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<ConnectToCastDeviceMutation, TError, ConnectToCastDeviceMutationVariables, TContext>) => {
    
    return useMutation<ConnectToCastDeviceMutation, TError, ConnectToCastDeviceMutationVariables, TContext>(
      {
    mutationKey: ['ConnectToCastDevice'],
    mutationFn: (variables?: ConnectToCastDeviceMutationVariables) => fetcher<ConnectToCastDeviceMutation, ConnectToCastDeviceMutationVariables>(ConnectToCastDeviceDocument, variables)(),
    ...options
  }
    )};

useConnectToCastDeviceMutation.getKey = () => ['ConnectToCastDevice'];


useConnectToCastDeviceMutation.fetcher = (variables: ConnectToCastDeviceMutationVariables, options?: RequestInit['headers']) => fetcher<ConnectToCastDeviceMutation, ConnectToCastDeviceMutationVariables>(ConnectToCastDeviceDocument, variables, options);

export const DisconnectFromCastDeviceDocument = `
    mutation DisconnectFromCastDevice {
  disconnectFromCastDevice {
    id
  }
}
    `;

export const useDisconnectFromCastDeviceMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<DisconnectFromCastDeviceMutation, TError, DisconnectFromCastDeviceMutationVariables, TContext>) => {
    
    return useMutation<DisconnectFromCastDeviceMutation, TError, DisconnectFromCastDeviceMutationVariables, TContext>(
      {
    mutationKey: ['DisconnectFromCastDevice'],
    mutationFn: (variables?: DisconnectFromCastDeviceMutationVariables) => fetcher<DisconnectFromCastDeviceMutation, DisconnectFromCastDeviceMutationVariables>(DisconnectFromCastDeviceDocument, variables)(),
    ...options
  }
    )};

useDisconnectFromCastDeviceMutation.getKey = () => ['DisconnectFromCastDevice'];


useDisconnectFromCastDeviceMutation.fetcher = (variables?: DisconnectFromCastDeviceMutationVariables, options?: RequestInit['headers']) => fetcher<DisconnectFromCastDeviceMutation, DisconnectFromCastDeviceMutationVariables>(DisconnectFromCastDeviceDocument, variables, options);

export const ListDevicesDocument = `
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

export const useListDevicesQuery = <
      TData = ListDevicesQuery,
      TError = unknown
    >(
      variables?: ListDevicesQueryVariables,
      options?: Omit<UseQueryOptions<ListDevicesQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<ListDevicesQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<ListDevicesQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['ListDevices'] : ['ListDevices', variables],
    queryFn: fetcher<ListDevicesQuery, ListDevicesQueryVariables>(ListDevicesDocument, variables),
    ...options
  }
    )};

useListDevicesQuery.getKey = (variables?: ListDevicesQueryVariables) => variables === undefined ? ['ListDevices'] : ['ListDevices', variables];

export const useInfiniteListDevicesQuery = <
      TData = InfiniteData<ListDevicesQuery>,
      TError = unknown
    >(
      variables: ListDevicesQueryVariables,
      options: Omit<UseInfiniteQueryOptions<ListDevicesQuery, TError, TData>, 'queryKey'> & { queryKey?: UseInfiniteQueryOptions<ListDevicesQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useInfiniteQuery<ListDevicesQuery, TError, TData>(
      (() => {
    const { queryKey: optionsQueryKey, ...restOptions } = options;
    return {
      queryKey: optionsQueryKey ?? variables === undefined ? ['ListDevices.infinite'] : ['ListDevices.infinite', variables],
      queryFn: (metaData) => fetcher<ListDevicesQuery, ListDevicesQueryVariables>(ListDevicesDocument, {...variables, ...(metaData.pageParam ?? {})})(),
      ...restOptions
    }
  })()
    )};

useInfiniteListDevicesQuery.getKey = (variables?: ListDevicesQueryVariables) => variables === undefined ? ['ListDevices.infinite'] : ['ListDevices.infinite', variables];


useListDevicesQuery.fetcher = (variables?: ListDevicesQueryVariables, options?: RequestInit['headers']) => fetcher<ListDevicesQuery, ListDevicesQueryVariables>(ListDevicesDocument, variables, options);

export const ListCastDevicesDocument = `
    query ListCastDevices {
  listCastDevices {
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

export const useListCastDevicesQuery = <
      TData = ListCastDevicesQuery,
      TError = unknown
    >(
      variables?: ListCastDevicesQueryVariables,
      options?: Omit<UseQueryOptions<ListCastDevicesQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<ListCastDevicesQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<ListCastDevicesQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['ListCastDevices'] : ['ListCastDevices', variables],
    queryFn: fetcher<ListCastDevicesQuery, ListCastDevicesQueryVariables>(ListCastDevicesDocument, variables),
    ...options
  }
    )};

useListCastDevicesQuery.getKey = (variables?: ListCastDevicesQueryVariables) => variables === undefined ? ['ListCastDevices'] : ['ListCastDevices', variables];

export const useInfiniteListCastDevicesQuery = <
      TData = InfiniteData<ListCastDevicesQuery>,
      TError = unknown
    >(
      variables: ListCastDevicesQueryVariables,
      options: Omit<UseInfiniteQueryOptions<ListCastDevicesQuery, TError, TData>, 'queryKey'> & { queryKey?: UseInfiniteQueryOptions<ListCastDevicesQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useInfiniteQuery<ListCastDevicesQuery, TError, TData>(
      (() => {
    const { queryKey: optionsQueryKey, ...restOptions } = options;
    return {
      queryKey: optionsQueryKey ?? variables === undefined ? ['ListCastDevices.infinite'] : ['ListCastDevices.infinite', variables],
      queryFn: (metaData) => fetcher<ListCastDevicesQuery, ListCastDevicesQueryVariables>(ListCastDevicesDocument, {...variables, ...(metaData.pageParam ?? {})})(),
      ...restOptions
    }
  })()
    )};

useInfiniteListCastDevicesQuery.getKey = (variables?: ListCastDevicesQueryVariables) => variables === undefined ? ['ListCastDevices.infinite'] : ['ListCastDevices.infinite', variables];


useListCastDevicesQuery.fetcher = (variables?: ListCastDevicesQueryVariables, options?: RequestInit['headers']) => fetcher<ListCastDevicesQuery, ListCastDevicesQueryVariables>(ListCastDevicesDocument, variables, options);

export const ConnectedDeviceDocument = `
    query ConnectedDevice {
  connectedDevice {
    id
    name
    app
    host
    port
    isConnected
  }
}
    `;

export const useConnectedDeviceQuery = <
      TData = ConnectedDeviceQuery,
      TError = unknown
    >(
      variables?: ConnectedDeviceQueryVariables,
      options?: Omit<UseQueryOptions<ConnectedDeviceQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<ConnectedDeviceQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<ConnectedDeviceQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['ConnectedDevice'] : ['ConnectedDevice', variables],
    queryFn: fetcher<ConnectedDeviceQuery, ConnectedDeviceQueryVariables>(ConnectedDeviceDocument, variables),
    ...options
  }
    )};

useConnectedDeviceQuery.getKey = (variables?: ConnectedDeviceQueryVariables) => variables === undefined ? ['ConnectedDevice'] : ['ConnectedDevice', variables];

export const useInfiniteConnectedDeviceQuery = <
      TData = InfiniteData<ConnectedDeviceQuery>,
      TError = unknown
    >(
      variables: ConnectedDeviceQueryVariables,
      options: Omit<UseInfiniteQueryOptions<ConnectedDeviceQuery, TError, TData>, 'queryKey'> & { queryKey?: UseInfiniteQueryOptions<ConnectedDeviceQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useInfiniteQuery<ConnectedDeviceQuery, TError, TData>(
      (() => {
    const { queryKey: optionsQueryKey, ...restOptions } = options;
    return {
      queryKey: optionsQueryKey ?? variables === undefined ? ['ConnectedDevice.infinite'] : ['ConnectedDevice.infinite', variables],
      queryFn: (metaData) => fetcher<ConnectedDeviceQuery, ConnectedDeviceQueryVariables>(ConnectedDeviceDocument, {...variables, ...(metaData.pageParam ?? {})})(),
      ...restOptions
    }
  })()
    )};

useInfiniteConnectedDeviceQuery.getKey = (variables?: ConnectedDeviceQueryVariables) => variables === undefined ? ['ConnectedDevice.infinite'] : ['ConnectedDevice.infinite', variables];


useConnectedDeviceQuery.fetcher = (variables?: ConnectedDeviceQueryVariables, options?: RequestInit['headers']) => fetcher<ConnectedDeviceQuery, ConnectedDeviceQueryVariables>(ConnectedDeviceDocument, variables, options);

export const ConnectedCastDeviceDocument = `
    query ConnectedCastDevice {
  connectedCastDevice {
    id
    name
    app
    host
    port
    isConnected
  }
}
    `;

export const useConnectedCastDeviceQuery = <
      TData = ConnectedCastDeviceQuery,
      TError = unknown
    >(
      variables?: ConnectedCastDeviceQueryVariables,
      options?: Omit<UseQueryOptions<ConnectedCastDeviceQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<ConnectedCastDeviceQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<ConnectedCastDeviceQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['ConnectedCastDevice'] : ['ConnectedCastDevice', variables],
    queryFn: fetcher<ConnectedCastDeviceQuery, ConnectedCastDeviceQueryVariables>(ConnectedCastDeviceDocument, variables),
    ...options
  }
    )};

useConnectedCastDeviceQuery.getKey = (variables?: ConnectedCastDeviceQueryVariables) => variables === undefined ? ['ConnectedCastDevice'] : ['ConnectedCastDevice', variables];

export const useInfiniteConnectedCastDeviceQuery = <
      TData = InfiniteData<ConnectedCastDeviceQuery>,
      TError = unknown
    >(
      variables: ConnectedCastDeviceQueryVariables,
      options: Omit<UseInfiniteQueryOptions<ConnectedCastDeviceQuery, TError, TData>, 'queryKey'> & { queryKey?: UseInfiniteQueryOptions<ConnectedCastDeviceQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useInfiniteQuery<ConnectedCastDeviceQuery, TError, TData>(
      (() => {
    const { queryKey: optionsQueryKey, ...restOptions } = options;
    return {
      queryKey: optionsQueryKey ?? variables === undefined ? ['ConnectedCastDevice.infinite'] : ['ConnectedCastDevice.infinite', variables],
      queryFn: (metaData) => fetcher<ConnectedCastDeviceQuery, ConnectedCastDeviceQueryVariables>(ConnectedCastDeviceDocument, {...variables, ...(metaData.pageParam ?? {})})(),
      ...restOptions
    }
  })()
    )};

useInfiniteConnectedCastDeviceQuery.getKey = (variables?: ConnectedCastDeviceQueryVariables) => variables === undefined ? ['ConnectedCastDevice.infinite'] : ['ConnectedCastDevice.infinite', variables];


useConnectedCastDeviceQuery.fetcher = (variables?: ConnectedCastDeviceQueryVariables, options?: RequestInit['headers']) => fetcher<ConnectedCastDeviceQuery, ConnectedCastDeviceQueryVariables>(ConnectedCastDeviceDocument, variables, options);

export const OnNewDeviceDocument = `
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
export const OnDeviceConnectedDocument = `
    subscription OnDeviceConnected {
  onConnected {
    id
    name
    service
    app
  }
}
    `;
export const OnDeviceDisconnectedDocument = `
    subscription OnDeviceDisconnected {
  onDisconnected {
    id
    name
    service
    app
  }
}
    `;
export const SetExtensionEnabledDocument = `
    mutation SetExtensionEnabled($id: ID!, $enabled: Boolean!) {
  setExtensionEnabled(id: $id, enabled: $enabled) {
    id
    name
    version
    author
    description
    homepage
    repository
    license
    logo
    topics
    capabilities
    allowedHosts
    libraryRead
    status
    path
  }
}
    `;

export const useSetExtensionEnabledMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<SetExtensionEnabledMutation, TError, SetExtensionEnabledMutationVariables, TContext>) => {
    
    return useMutation<SetExtensionEnabledMutation, TError, SetExtensionEnabledMutationVariables, TContext>(
      {
    mutationKey: ['SetExtensionEnabled'],
    mutationFn: (variables?: SetExtensionEnabledMutationVariables) => fetcher<SetExtensionEnabledMutation, SetExtensionEnabledMutationVariables>(SetExtensionEnabledDocument, variables)(),
    ...options
  }
    )};

useSetExtensionEnabledMutation.getKey = () => ['SetExtensionEnabled'];


useSetExtensionEnabledMutation.fetcher = (variables: SetExtensionEnabledMutationVariables, options?: RequestInit['headers']) => fetcher<SetExtensionEnabledMutation, SetExtensionEnabledMutationVariables>(SetExtensionEnabledDocument, variables, options);

export const RescanExtensionsDocument = `
    mutation RescanExtensions($filter: String) {
  rescanExtensions(filter: $filter) {
    id
    name
    version
    author
    description
    homepage
    repository
    license
    logo
    topics
    capabilities
    allowedHosts
    libraryRead
    status
    path
  }
}
    `;

export const useRescanExtensionsMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<RescanExtensionsMutation, TError, RescanExtensionsMutationVariables, TContext>) => {
    
    return useMutation<RescanExtensionsMutation, TError, RescanExtensionsMutationVariables, TContext>(
      {
    mutationKey: ['RescanExtensions'],
    mutationFn: (variables?: RescanExtensionsMutationVariables) => fetcher<RescanExtensionsMutation, RescanExtensionsMutationVariables>(RescanExtensionsDocument, variables)(),
    ...options
  }
    )};

useRescanExtensionsMutation.getKey = () => ['RescanExtensions'];


useRescanExtensionsMutation.fetcher = (variables?: RescanExtensionsMutationVariables, options?: RequestInit['headers']) => fetcher<RescanExtensionsMutation, RescanExtensionsMutationVariables>(RescanExtensionsDocument, variables, options);

export const GetExtensionsDocument = `
    query GetExtensions($filter: String) {
  extensions(filter: $filter) {
    id
    name
    version
    author
    description
    homepage
    repository
    license
    logo
    topics
    capabilities
    allowedHosts
    libraryRead
    status
    path
  }
}
    `;

export const useGetExtensionsQuery = <
      TData = GetExtensionsQuery,
      TError = unknown
    >(
      variables?: GetExtensionsQueryVariables,
      options?: Omit<UseQueryOptions<GetExtensionsQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetExtensionsQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetExtensionsQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['GetExtensions'] : ['GetExtensions', variables],
    queryFn: fetcher<GetExtensionsQuery, GetExtensionsQueryVariables>(GetExtensionsDocument, variables),
    ...options
  }
    )};

useGetExtensionsQuery.getKey = (variables?: GetExtensionsQueryVariables) => variables === undefined ? ['GetExtensions'] : ['GetExtensions', variables];

export const useInfiniteGetExtensionsQuery = <
      TData = InfiniteData<GetExtensionsQuery>,
      TError = unknown
    >(
      variables: GetExtensionsQueryVariables,
      options: Omit<UseInfiniteQueryOptions<GetExtensionsQuery, TError, TData>, 'queryKey'> & { queryKey?: UseInfiniteQueryOptions<GetExtensionsQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useInfiniteQuery<GetExtensionsQuery, TError, TData>(
      (() => {
    const { queryKey: optionsQueryKey, ...restOptions } = options;
    return {
      queryKey: optionsQueryKey ?? variables === undefined ? ['GetExtensions.infinite'] : ['GetExtensions.infinite', variables],
      queryFn: (metaData) => fetcher<GetExtensionsQuery, GetExtensionsQueryVariables>(GetExtensionsDocument, {...variables, ...(metaData.pageParam ?? {})})(),
      ...restOptions
    }
  })()
    )};

useInfiniteGetExtensionsQuery.getKey = (variables?: GetExtensionsQueryVariables) => variables === undefined ? ['GetExtensions.infinite'] : ['GetExtensions.infinite', variables];


useGetExtensionsQuery.fetcher = (variables?: GetExtensionsQueryVariables, options?: RequestInit['headers']) => fetcher<GetExtensionsQuery, GetExtensionsQueryVariables>(GetExtensionsDocument, variables, options);

export const LikeTrackDocument = `
    mutation LikeTrack($id: String!, $like: Boolean!) {
  likeTrack(id: $id, like: $like)
}
    `;

export const useLikeTrackMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<LikeTrackMutation, TError, LikeTrackMutationVariables, TContext>) => {
    
    return useMutation<LikeTrackMutation, TError, LikeTrackMutationVariables, TContext>(
      {
    mutationKey: ['LikeTrack'],
    mutationFn: (variables?: LikeTrackMutationVariables) => fetcher<LikeTrackMutation, LikeTrackMutationVariables>(LikeTrackDocument, variables)(),
    ...options
  }
    )};

useLikeTrackMutation.getKey = () => ['LikeTrack'];


useLikeTrackMutation.fetcher = (variables: LikeTrackMutationVariables, options?: RequestInit['headers']) => fetcher<LikeTrackMutation, LikeTrackMutationVariables>(LikeTrackDocument, variables, options);

export const GetAlbumsDocument = `
    query GetAlbums($filter: String, $offset: Int, $limit: Int) {
  albums(filter: $filter, offset: $offset, limit: $limit) {
    ...AlbumFragment
    tracks {
      ...TrackFragment
    }
  }
}
    ${AlbumFragmentFragmentDoc}
${TrackFragmentFragmentDoc}`;

export const useGetAlbumsQuery = <
      TData = GetAlbumsQuery,
      TError = unknown
    >(
      variables?: GetAlbumsQueryVariables,
      options?: Omit<UseQueryOptions<GetAlbumsQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetAlbumsQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetAlbumsQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['GetAlbums'] : ['GetAlbums', variables],
    queryFn: fetcher<GetAlbumsQuery, GetAlbumsQueryVariables>(GetAlbumsDocument, variables),
    ...options
  }
    )};

useGetAlbumsQuery.getKey = (variables?: GetAlbumsQueryVariables) => variables === undefined ? ['GetAlbums'] : ['GetAlbums', variables];

export const useInfiniteGetAlbumsQuery = <
      TData = InfiniteData<GetAlbumsQuery>,
      TError = unknown
    >(
      variables: GetAlbumsQueryVariables,
      options: Omit<UseInfiniteQueryOptions<GetAlbumsQuery, TError, TData>, 'queryKey'> & { queryKey?: UseInfiniteQueryOptions<GetAlbumsQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useInfiniteQuery<GetAlbumsQuery, TError, TData>(
      (() => {
    const { queryKey: optionsQueryKey, ...restOptions } = options;
    return {
      queryKey: optionsQueryKey ?? variables === undefined ? ['GetAlbums.infinite'] : ['GetAlbums.infinite', variables],
      queryFn: (metaData) => fetcher<GetAlbumsQuery, GetAlbumsQueryVariables>(GetAlbumsDocument, {...variables, ...(metaData.pageParam ?? {})})(),
      ...restOptions
    }
  })()
    )};

useInfiniteGetAlbumsQuery.getKey = (variables?: GetAlbumsQueryVariables) => variables === undefined ? ['GetAlbums.infinite'] : ['GetAlbums.infinite', variables];


useGetAlbumsQuery.fetcher = (variables?: GetAlbumsQueryVariables, options?: RequestInit['headers']) => fetcher<GetAlbumsQuery, GetAlbumsQueryVariables>(GetAlbumsDocument, variables, options);

export const GetArtistsDocument = `
    query GetArtists($filter: String, $offset: Int, $limit: Int) {
  artists(filter: $filter, offset: $offset, limit: $limit) {
    ...ArtistFragment
  }
}
    ${ArtistFragmentFragmentDoc}`;

export const useGetArtistsQuery = <
      TData = GetArtistsQuery,
      TError = unknown
    >(
      variables?: GetArtistsQueryVariables,
      options?: Omit<UseQueryOptions<GetArtistsQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetArtistsQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetArtistsQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['GetArtists'] : ['GetArtists', variables],
    queryFn: fetcher<GetArtistsQuery, GetArtistsQueryVariables>(GetArtistsDocument, variables),
    ...options
  }
    )};

useGetArtistsQuery.getKey = (variables?: GetArtistsQueryVariables) => variables === undefined ? ['GetArtists'] : ['GetArtists', variables];

export const useInfiniteGetArtistsQuery = <
      TData = InfiniteData<GetArtistsQuery>,
      TError = unknown
    >(
      variables: GetArtistsQueryVariables,
      options: Omit<UseInfiniteQueryOptions<GetArtistsQuery, TError, TData>, 'queryKey'> & { queryKey?: UseInfiniteQueryOptions<GetArtistsQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useInfiniteQuery<GetArtistsQuery, TError, TData>(
      (() => {
    const { queryKey: optionsQueryKey, ...restOptions } = options;
    return {
      queryKey: optionsQueryKey ?? variables === undefined ? ['GetArtists.infinite'] : ['GetArtists.infinite', variables],
      queryFn: (metaData) => fetcher<GetArtistsQuery, GetArtistsQueryVariables>(GetArtistsDocument, {...variables, ...(metaData.pageParam ?? {})})(),
      ...restOptions
    }
  })()
    )};

useInfiniteGetArtistsQuery.getKey = (variables?: GetArtistsQueryVariables) => variables === undefined ? ['GetArtists.infinite'] : ['GetArtists.infinite', variables];


useGetArtistsQuery.fetcher = (variables?: GetArtistsQueryVariables, options?: RequestInit['headers']) => fetcher<GetArtistsQuery, GetArtistsQueryVariables>(GetArtistsDocument, variables, options);

export const GetTracksDocument = `
    query GetTracks($filter: String, $offset: Int, $limit: Int) {
  tracks(filter: $filter, offset: $offset, limit: $limit) {
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

export const useGetTracksQuery = <
      TData = GetTracksQuery,
      TError = unknown
    >(
      variables?: GetTracksQueryVariables,
      options?: Omit<UseQueryOptions<GetTracksQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetTracksQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetTracksQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['GetTracks'] : ['GetTracks', variables],
    queryFn: fetcher<GetTracksQuery, GetTracksQueryVariables>(GetTracksDocument, variables),
    ...options
  }
    )};

useGetTracksQuery.getKey = (variables?: GetTracksQueryVariables) => variables === undefined ? ['GetTracks'] : ['GetTracks', variables];

export const useInfiniteGetTracksQuery = <
      TData = InfiniteData<GetTracksQuery>,
      TError = unknown
    >(
      variables: GetTracksQueryVariables,
      options: Omit<UseInfiniteQueryOptions<GetTracksQuery, TError, TData>, 'queryKey'> & { queryKey?: UseInfiniteQueryOptions<GetTracksQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useInfiniteQuery<GetTracksQuery, TError, TData>(
      (() => {
    const { queryKey: optionsQueryKey, ...restOptions } = options;
    return {
      queryKey: optionsQueryKey ?? variables === undefined ? ['GetTracks.infinite'] : ['GetTracks.infinite', variables],
      queryFn: (metaData) => fetcher<GetTracksQuery, GetTracksQueryVariables>(GetTracksDocument, {...variables, ...(metaData.pageParam ?? {})})(),
      ...restOptions
    }
  })()
    )};

useInfiniteGetTracksQuery.getKey = (variables?: GetTracksQueryVariables) => variables === undefined ? ['GetTracks.infinite'] : ['GetTracks.infinite', variables];


useGetTracksQuery.fetcher = (variables?: GetTracksQueryVariables, options?: RequestInit['headers']) => fetcher<GetTracksQuery, GetTracksQueryVariables>(GetTracksDocument, variables, options);

export const GetArtistDocument = `
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

export const useGetArtistQuery = <
      TData = GetArtistQuery,
      TError = unknown
    >(
      variables: GetArtistQueryVariables,
      options?: Omit<UseQueryOptions<GetArtistQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetArtistQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetArtistQuery, TError, TData>(
      {
    queryKey: ['GetArtist', variables],
    queryFn: fetcher<GetArtistQuery, GetArtistQueryVariables>(GetArtistDocument, variables),
    ...options
  }
    )};

useGetArtistQuery.getKey = (variables: GetArtistQueryVariables) => ['GetArtist', variables];

export const useInfiniteGetArtistQuery = <
      TData = InfiniteData<GetArtistQuery>,
      TError = unknown
    >(
      variables: GetArtistQueryVariables,
      options: Omit<UseInfiniteQueryOptions<GetArtistQuery, TError, TData>, 'queryKey'> & { queryKey?: UseInfiniteQueryOptions<GetArtistQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useInfiniteQuery<GetArtistQuery, TError, TData>(
      (() => {
    const { queryKey: optionsQueryKey, ...restOptions } = options;
    return {
      queryKey: optionsQueryKey ?? ['GetArtist.infinite', variables],
      queryFn: (metaData) => fetcher<GetArtistQuery, GetArtistQueryVariables>(GetArtistDocument, {...variables, ...(metaData.pageParam ?? {})})(),
      ...restOptions
    }
  })()
    )};

useInfiniteGetArtistQuery.getKey = (variables: GetArtistQueryVariables) => ['GetArtist.infinite', variables];


useGetArtistQuery.fetcher = (variables: GetArtistQueryVariables, options?: RequestInit['headers']) => fetcher<GetArtistQuery, GetArtistQueryVariables>(GetArtistDocument, variables, options);

export const GetAlbumDocument = `
    query GetAlbum($id: ID!) {
  album(id: $id) {
    ...AlbumFragment
    tracks {
      id
      trackNumber
      discNumber
      title
      artist
      artists {
        id
        name
      }
      duration
      uri
    }
  }
}
    ${AlbumFragmentFragmentDoc}`;

export const useGetAlbumQuery = <
      TData = GetAlbumQuery,
      TError = unknown
    >(
      variables: GetAlbumQueryVariables,
      options?: Omit<UseQueryOptions<GetAlbumQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetAlbumQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetAlbumQuery, TError, TData>(
      {
    queryKey: ['GetAlbum', variables],
    queryFn: fetcher<GetAlbumQuery, GetAlbumQueryVariables>(GetAlbumDocument, variables),
    ...options
  }
    )};

useGetAlbumQuery.getKey = (variables: GetAlbumQueryVariables) => ['GetAlbum', variables];

export const useInfiniteGetAlbumQuery = <
      TData = InfiniteData<GetAlbumQuery>,
      TError = unknown
    >(
      variables: GetAlbumQueryVariables,
      options: Omit<UseInfiniteQueryOptions<GetAlbumQuery, TError, TData>, 'queryKey'> & { queryKey?: UseInfiniteQueryOptions<GetAlbumQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useInfiniteQuery<GetAlbumQuery, TError, TData>(
      (() => {
    const { queryKey: optionsQueryKey, ...restOptions } = options;
    return {
      queryKey: optionsQueryKey ?? ['GetAlbum.infinite', variables],
      queryFn: (metaData) => fetcher<GetAlbumQuery, GetAlbumQueryVariables>(GetAlbumDocument, {...variables, ...(metaData.pageParam ?? {})})(),
      ...restOptions
    }
  })()
    )};

useInfiniteGetAlbumQuery.getKey = (variables: GetAlbumQueryVariables) => ['GetAlbum.infinite', variables];


useGetAlbumQuery.fetcher = (variables: GetAlbumQueryVariables, options?: RequestInit['headers']) => fetcher<GetAlbumQuery, GetAlbumQueryVariables>(GetAlbumDocument, variables, options);

export const SearchDocument = `
    query Search($keyword: String!) {
  search(keyword: $keyword) {
    artists {
      id
      name
      picture
      source
    }
    albums {
      id
      title
      artist
      cover
      source
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
      source
    }
  }
}
    `;

export const useSearchQuery = <
      TData = SearchQuery,
      TError = unknown
    >(
      variables: SearchQueryVariables,
      options?: Omit<UseQueryOptions<SearchQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<SearchQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<SearchQuery, TError, TData>(
      {
    queryKey: ['Search', variables],
    queryFn: fetcher<SearchQuery, SearchQueryVariables>(SearchDocument, variables),
    ...options
  }
    )};

useSearchQuery.getKey = (variables: SearchQueryVariables) => ['Search', variables];

export const useInfiniteSearchQuery = <
      TData = InfiniteData<SearchQuery>,
      TError = unknown
    >(
      variables: SearchQueryVariables,
      options: Omit<UseInfiniteQueryOptions<SearchQuery, TError, TData>, 'queryKey'> & { queryKey?: UseInfiniteQueryOptions<SearchQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useInfiniteQuery<SearchQuery, TError, TData>(
      (() => {
    const { queryKey: optionsQueryKey, ...restOptions } = options;
    return {
      queryKey: optionsQueryKey ?? ['Search.infinite', variables],
      queryFn: (metaData) => fetcher<SearchQuery, SearchQueryVariables>(SearchDocument, {...variables, ...(metaData.pageParam ?? {})})(),
      ...restOptions
    }
  })()
    )};

useInfiniteSearchQuery.getKey = (variables: SearchQueryVariables) => ['Search.infinite', variables];


useSearchQuery.fetcher = (variables: SearchQueryVariables, options?: RequestInit['headers']) => fetcher<SearchQuery, SearchQueryVariables>(SearchDocument, variables, options);

export const GetGenresDocument = `
    query GetGenres($offset: Int, $limit: Int) {
  genres(offset: $offset, limit: $limit) {
    id
    name
    trackCount
  }
}
    `;

export const useGetGenresQuery = <
      TData = GetGenresQuery,
      TError = unknown
    >(
      variables?: GetGenresQueryVariables,
      options?: Omit<UseQueryOptions<GetGenresQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetGenresQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetGenresQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['GetGenres'] : ['GetGenres', variables],
    queryFn: fetcher<GetGenresQuery, GetGenresQueryVariables>(GetGenresDocument, variables),
    ...options
  }
    )};

useGetGenresQuery.getKey = (variables?: GetGenresQueryVariables) => variables === undefined ? ['GetGenres'] : ['GetGenres', variables];

export const useInfiniteGetGenresQuery = <
      TData = InfiniteData<GetGenresQuery>,
      TError = unknown
    >(
      variables: GetGenresQueryVariables,
      options: Omit<UseInfiniteQueryOptions<GetGenresQuery, TError, TData>, 'queryKey'> & { queryKey?: UseInfiniteQueryOptions<GetGenresQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useInfiniteQuery<GetGenresQuery, TError, TData>(
      (() => {
    const { queryKey: optionsQueryKey, ...restOptions } = options;
    return {
      queryKey: optionsQueryKey ?? variables === undefined ? ['GetGenres.infinite'] : ['GetGenres.infinite', variables],
      queryFn: (metaData) => fetcher<GetGenresQuery, GetGenresQueryVariables>(GetGenresDocument, {...variables, ...(metaData.pageParam ?? {})})(),
      ...restOptions
    }
  })()
    )};

useInfiniteGetGenresQuery.getKey = (variables?: GetGenresQueryVariables) => variables === undefined ? ['GetGenres.infinite'] : ['GetGenres.infinite', variables];


useGetGenresQuery.fetcher = (variables?: GetGenresQueryVariables, options?: RequestInit['headers']) => fetcher<GetGenresQuery, GetGenresQueryVariables>(GetGenresDocument, variables, options);

export const GetGenreTracksDocument = `
    query GetGenreTracks($id: ID!, $offset: Int, $limit: Int) {
  genreTracks(id: $id, offset: $offset, limit: $limit) {
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
    `;

export const useGetGenreTracksQuery = <
      TData = GetGenreTracksQuery,
      TError = unknown
    >(
      variables: GetGenreTracksQueryVariables,
      options?: Omit<UseQueryOptions<GetGenreTracksQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetGenreTracksQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetGenreTracksQuery, TError, TData>(
      {
    queryKey: ['GetGenreTracks', variables],
    queryFn: fetcher<GetGenreTracksQuery, GetGenreTracksQueryVariables>(GetGenreTracksDocument, variables),
    ...options
  }
    )};

useGetGenreTracksQuery.getKey = (variables: GetGenreTracksQueryVariables) => ['GetGenreTracks', variables];

export const useInfiniteGetGenreTracksQuery = <
      TData = InfiniteData<GetGenreTracksQuery>,
      TError = unknown
    >(
      variables: GetGenreTracksQueryVariables,
      options: Omit<UseInfiniteQueryOptions<GetGenreTracksQuery, TError, TData>, 'queryKey'> & { queryKey?: UseInfiniteQueryOptions<GetGenreTracksQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useInfiniteQuery<GetGenreTracksQuery, TError, TData>(
      (() => {
    const { queryKey: optionsQueryKey, ...restOptions } = options;
    return {
      queryKey: optionsQueryKey ?? ['GetGenreTracks.infinite', variables],
      queryFn: (metaData) => fetcher<GetGenreTracksQuery, GetGenreTracksQueryVariables>(GetGenreTracksDocument, {...variables, ...(metaData.pageParam ?? {})})(),
      ...restOptions
    }
  })()
    )};

useInfiniteGetGenreTracksQuery.getKey = (variables: GetGenreTracksQueryVariables) => ['GetGenreTracks.infinite', variables];


useGetGenreTracksQuery.fetcher = (variables: GetGenreTracksQueryVariables, options?: RequestInit['headers']) => fetcher<GetGenreTracksQuery, GetGenreTracksQueryVariables>(GetGenreTracksDocument, variables, options);

export const SetAudioSettingDocument = `
    mutation SetAudioSetting($name: String!, $value: Int!) {
  setAudioSetting(name: $name, value: $value) {
    ...AudioSettingsFields
  }
}
    ${AudioSettingsFieldsFragmentDoc}`;

export const useSetAudioSettingMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<SetAudioSettingMutation, TError, SetAudioSettingMutationVariables, TContext>) => {
    
    return useMutation<SetAudioSettingMutation, TError, SetAudioSettingMutationVariables, TContext>(
      {
    mutationKey: ['SetAudioSetting'],
    mutationFn: (variables?: SetAudioSettingMutationVariables) => fetcher<SetAudioSettingMutation, SetAudioSettingMutationVariables>(SetAudioSettingDocument, variables)(),
    ...options
  }
    )};

useSetAudioSettingMutation.getKey = () => ['SetAudioSetting'];


useSetAudioSettingMutation.fetcher = (variables: SetAudioSettingMutationVariables, options?: RequestInit['headers']) => fetcher<SetAudioSettingMutation, SetAudioSettingMutationVariables>(SetAudioSettingDocument, variables, options);

export const SetEqBandGainDocument = `
    mutation SetEqBandGain($band: Int!, $gain: Int!) {
  setEqBandGain(band: $band, gain: $gain) {
    ...AudioSettingsFields
  }
}
    ${AudioSettingsFieldsFragmentDoc}`;

export const useSetEqBandGainMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<SetEqBandGainMutation, TError, SetEqBandGainMutationVariables, TContext>) => {
    
    return useMutation<SetEqBandGainMutation, TError, SetEqBandGainMutationVariables, TContext>(
      {
    mutationKey: ['SetEqBandGain'],
    mutationFn: (variables?: SetEqBandGainMutationVariables) => fetcher<SetEqBandGainMutation, SetEqBandGainMutationVariables>(SetEqBandGainDocument, variables)(),
    ...options
  }
    )};

useSetEqBandGainMutation.getKey = () => ['SetEqBandGain'];


useSetEqBandGainMutation.fetcher = (variables: SetEqBandGainMutationVariables, options?: RequestInit['headers']) => fetcher<SetEqBandGainMutation, SetEqBandGainMutationVariables>(SetEqBandGainDocument, variables, options);

export const GetAudioSettingsDocument = `
    query GetAudioSettings {
  audioSettings {
    ...AudioSettingsFields
  }
}
    ${AudioSettingsFieldsFragmentDoc}`;

export const useGetAudioSettingsQuery = <
      TData = GetAudioSettingsQuery,
      TError = unknown
    >(
      variables?: GetAudioSettingsQueryVariables,
      options?: Omit<UseQueryOptions<GetAudioSettingsQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetAudioSettingsQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetAudioSettingsQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['GetAudioSettings'] : ['GetAudioSettings', variables],
    queryFn: fetcher<GetAudioSettingsQuery, GetAudioSettingsQueryVariables>(GetAudioSettingsDocument, variables),
    ...options
  }
    )};

useGetAudioSettingsQuery.getKey = (variables?: GetAudioSettingsQueryVariables) => variables === undefined ? ['GetAudioSettings'] : ['GetAudioSettings', variables];

export const useInfiniteGetAudioSettingsQuery = <
      TData = InfiniteData<GetAudioSettingsQuery>,
      TError = unknown
    >(
      variables: GetAudioSettingsQueryVariables,
      options: Omit<UseInfiniteQueryOptions<GetAudioSettingsQuery, TError, TData>, 'queryKey'> & { queryKey?: UseInfiniteQueryOptions<GetAudioSettingsQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useInfiniteQuery<GetAudioSettingsQuery, TError, TData>(
      (() => {
    const { queryKey: optionsQueryKey, ...restOptions } = options;
    return {
      queryKey: optionsQueryKey ?? variables === undefined ? ['GetAudioSettings.infinite'] : ['GetAudioSettings.infinite', variables],
      queryFn: (metaData) => fetcher<GetAudioSettingsQuery, GetAudioSettingsQueryVariables>(GetAudioSettingsDocument, {...variables, ...(metaData.pageParam ?? {})})(),
      ...restOptions
    }
  })()
    )};

useInfiniteGetAudioSettingsQuery.getKey = (variables?: GetAudioSettingsQueryVariables) => variables === undefined ? ['GetAudioSettings.infinite'] : ['GetAudioSettings.infinite', variables];


useGetAudioSettingsQuery.fetcher = (variables?: GetAudioSettingsQueryVariables, options?: RequestInit['headers']) => fetcher<GetAudioSettingsQuery, GetAudioSettingsQueryVariables>(GetAudioSettingsDocument, variables, options);

export const NextDocument = `
    mutation Next {
  next
}
    `;

export const useNextMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<NextMutation, TError, NextMutationVariables, TContext>) => {
    
    return useMutation<NextMutation, TError, NextMutationVariables, TContext>(
      {
    mutationKey: ['Next'],
    mutationFn: (variables?: NextMutationVariables) => fetcher<NextMutation, NextMutationVariables>(NextDocument, variables)(),
    ...options
  }
    )};

useNextMutation.getKey = () => ['Next'];


useNextMutation.fetcher = (variables?: NextMutationVariables, options?: RequestInit['headers']) => fetcher<NextMutation, NextMutationVariables>(NextDocument, variables, options);

export const PauseDocument = `
    mutation Pause {
  pause
}
    `;

export const usePauseMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<PauseMutation, TError, PauseMutationVariables, TContext>) => {
    
    return useMutation<PauseMutation, TError, PauseMutationVariables, TContext>(
      {
    mutationKey: ['Pause'],
    mutationFn: (variables?: PauseMutationVariables) => fetcher<PauseMutation, PauseMutationVariables>(PauseDocument, variables)(),
    ...options
  }
    )};

usePauseMutation.getKey = () => ['Pause'];


usePauseMutation.fetcher = (variables?: PauseMutationVariables, options?: RequestInit['headers']) => fetcher<PauseMutation, PauseMutationVariables>(PauseDocument, variables, options);

export const PlayDocument = `
    mutation Play {
  play
}
    `;

export const usePlayMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<PlayMutation, TError, PlayMutationVariables, TContext>) => {
    
    return useMutation<PlayMutation, TError, PlayMutationVariables, TContext>(
      {
    mutationKey: ['Play'],
    mutationFn: (variables?: PlayMutationVariables) => fetcher<PlayMutation, PlayMutationVariables>(PlayDocument, variables)(),
    ...options
  }
    )};

usePlayMutation.getKey = () => ['Play'];


usePlayMutation.fetcher = (variables?: PlayMutationVariables, options?: RequestInit['headers']) => fetcher<PlayMutation, PlayMutationVariables>(PlayDocument, variables, options);

export const PreviousDocument = `
    mutation Previous {
  previous
}
    `;

export const usePreviousMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<PreviousMutation, TError, PreviousMutationVariables, TContext>) => {
    
    return useMutation<PreviousMutation, TError, PreviousMutationVariables, TContext>(
      {
    mutationKey: ['Previous'],
    mutationFn: (variables?: PreviousMutationVariables) => fetcher<PreviousMutation, PreviousMutationVariables>(PreviousDocument, variables)(),
    ...options
  }
    )};

usePreviousMutation.getKey = () => ['Previous'];


usePreviousMutation.fetcher = (variables?: PreviousMutationVariables, options?: RequestInit['headers']) => fetcher<PreviousMutation, PreviousMutationVariables>(PreviousDocument, variables, options);

export const SeekDocument = `
    mutation Seek($position: Int!) {
  seek(position: $position)
}
    `;

export const useSeekMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<SeekMutation, TError, SeekMutationVariables, TContext>) => {
    
    return useMutation<SeekMutation, TError, SeekMutationVariables, TContext>(
      {
    mutationKey: ['Seek'],
    mutationFn: (variables?: SeekMutationVariables) => fetcher<SeekMutation, SeekMutationVariables>(SeekDocument, variables)(),
    ...options
  }
    )};

useSeekMutation.getKey = () => ['Seek'];


useSeekMutation.fetcher = (variables: SeekMutationVariables, options?: RequestInit['headers']) => fetcher<SeekMutation, SeekMutationVariables>(SeekDocument, variables, options);

export const CurrentlyPlayingSongDocument = `
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
        cover
      }
    }
    index
    isPlaying
    positionMs
  }
}
    ${TrackFragmentFragmentDoc}`;

export const useCurrentlyPlayingSongQuery = <
      TData = CurrentlyPlayingSongQuery,
      TError = unknown
    >(
      variables?: CurrentlyPlayingSongQueryVariables,
      options?: Omit<UseQueryOptions<CurrentlyPlayingSongQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<CurrentlyPlayingSongQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<CurrentlyPlayingSongQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['CurrentlyPlayingSong'] : ['CurrentlyPlayingSong', variables],
    queryFn: fetcher<CurrentlyPlayingSongQuery, CurrentlyPlayingSongQueryVariables>(CurrentlyPlayingSongDocument, variables),
    ...options
  }
    )};

useCurrentlyPlayingSongQuery.getKey = (variables?: CurrentlyPlayingSongQueryVariables) => variables === undefined ? ['CurrentlyPlayingSong'] : ['CurrentlyPlayingSong', variables];

export const useInfiniteCurrentlyPlayingSongQuery = <
      TData = InfiniteData<CurrentlyPlayingSongQuery>,
      TError = unknown
    >(
      variables: CurrentlyPlayingSongQueryVariables,
      options: Omit<UseInfiniteQueryOptions<CurrentlyPlayingSongQuery, TError, TData>, 'queryKey'> & { queryKey?: UseInfiniteQueryOptions<CurrentlyPlayingSongQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useInfiniteQuery<CurrentlyPlayingSongQuery, TError, TData>(
      (() => {
    const { queryKey: optionsQueryKey, ...restOptions } = options;
    return {
      queryKey: optionsQueryKey ?? variables === undefined ? ['CurrentlyPlayingSong.infinite'] : ['CurrentlyPlayingSong.infinite', variables],
      queryFn: (metaData) => fetcher<CurrentlyPlayingSongQuery, CurrentlyPlayingSongQueryVariables>(CurrentlyPlayingSongDocument, {...variables, ...(metaData.pageParam ?? {})})(),
      ...restOptions
    }
  })()
    )};

useInfiniteCurrentlyPlayingSongQuery.getKey = (variables?: CurrentlyPlayingSongQueryVariables) => variables === undefined ? ['CurrentlyPlayingSong.infinite'] : ['CurrentlyPlayingSong.infinite', variables];


useCurrentlyPlayingSongQuery.fetcher = (variables?: CurrentlyPlayingSongQueryVariables, options?: RequestInit['headers']) => fetcher<CurrentlyPlayingSongQuery, CurrentlyPlayingSongQueryVariables>(CurrentlyPlayingSongDocument, variables, options);

export const PlayerStateChangedDocument = `
    subscription PlayerStateChanged {
  playerState {
    isPlaying
  }
}
    `;
export const TrackTimePositionChangedDocument = `
    subscription TrackTimePositionChanged {
  trackTimePosition {
    positionMs
  }
}
    `;
export const CurrentlyPlayingSongChangedDocument = `
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
    ${TrackFragmentFragmentDoc}`;
export const OnLevelsDocument = `
    subscription OnLevels {
  levels {
    left
    right
    lowLeft
    lowRight
  }
}
    `;
export const CreatePlaylistDocument = `
    mutation CreatePlaylist($name: String!, $description: String, $smart: SmartPlaylistInput) {
  createPlaylist(name: $name, description: $description, smart: $smart) {
    id
    name
    description
    isSmart
    rsql
  }
}
    `;

export const useCreatePlaylistMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<CreatePlaylistMutation, TError, CreatePlaylistMutationVariables, TContext>) => {
    
    return useMutation<CreatePlaylistMutation, TError, CreatePlaylistMutationVariables, TContext>(
      {
    mutationKey: ['CreatePlaylist'],
    mutationFn: (variables?: CreatePlaylistMutationVariables) => fetcher<CreatePlaylistMutation, CreatePlaylistMutationVariables>(CreatePlaylistDocument, variables)(),
    ...options
  }
    )};

useCreatePlaylistMutation.getKey = () => ['CreatePlaylist'];


useCreatePlaylistMutation.fetcher = (variables: CreatePlaylistMutationVariables, options?: RequestInit['headers']) => fetcher<CreatePlaylistMutation, CreatePlaylistMutationVariables>(CreatePlaylistDocument, variables, options);

export const DeletePlaylistDocument = `
    mutation DeletePlaylist($id: ID!) {
  deletePlaylist(id: $id) {
    id
  }
}
    `;

export const useDeletePlaylistMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<DeletePlaylistMutation, TError, DeletePlaylistMutationVariables, TContext>) => {
    
    return useMutation<DeletePlaylistMutation, TError, DeletePlaylistMutationVariables, TContext>(
      {
    mutationKey: ['DeletePlaylist'],
    mutationFn: (variables?: DeletePlaylistMutationVariables) => fetcher<DeletePlaylistMutation, DeletePlaylistMutationVariables>(DeletePlaylistDocument, variables)(),
    ...options
  }
    )};

useDeletePlaylistMutation.getKey = () => ['DeletePlaylist'];


useDeletePlaylistMutation.fetcher = (variables: DeletePlaylistMutationVariables, options?: RequestInit['headers']) => fetcher<DeletePlaylistMutation, DeletePlaylistMutationVariables>(DeletePlaylistDocument, variables, options);

export const AddTrackToPlaylistDocument = `
    mutation AddTrackToPlaylist($playlistId: ID!, $trackId: ID!) {
  addTrackToPlaylist(id: $playlistId, trackId: $trackId) {
    ...PlaylistFragment
  }
}
    ${PlaylistFragmentFragmentDoc}`;

export const useAddTrackToPlaylistMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<AddTrackToPlaylistMutation, TError, AddTrackToPlaylistMutationVariables, TContext>) => {
    
    return useMutation<AddTrackToPlaylistMutation, TError, AddTrackToPlaylistMutationVariables, TContext>(
      {
    mutationKey: ['AddTrackToPlaylist'],
    mutationFn: (variables?: AddTrackToPlaylistMutationVariables) => fetcher<AddTrackToPlaylistMutation, AddTrackToPlaylistMutationVariables>(AddTrackToPlaylistDocument, variables)(),
    ...options
  }
    )};

useAddTrackToPlaylistMutation.getKey = () => ['AddTrackToPlaylist'];


useAddTrackToPlaylistMutation.fetcher = (variables: AddTrackToPlaylistMutationVariables, options?: RequestInit['headers']) => fetcher<AddTrackToPlaylistMutation, AddTrackToPlaylistMutationVariables>(AddTrackToPlaylistDocument, variables, options);

export const RemoveTrackFromPlaylistDocument = `
    mutation RemoveTrackFromPlaylist($playlistId: ID!, $position: Int!) {
  removeTrackFromPlaylist(id: $playlistId, position: $position) {
    ...PlaylistFragment
  }
}
    ${PlaylistFragmentFragmentDoc}`;

export const useRemoveTrackFromPlaylistMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<RemoveTrackFromPlaylistMutation, TError, RemoveTrackFromPlaylistMutationVariables, TContext>) => {
    
    return useMutation<RemoveTrackFromPlaylistMutation, TError, RemoveTrackFromPlaylistMutationVariables, TContext>(
      {
    mutationKey: ['RemoveTrackFromPlaylist'],
    mutationFn: (variables?: RemoveTrackFromPlaylistMutationVariables) => fetcher<RemoveTrackFromPlaylistMutation, RemoveTrackFromPlaylistMutationVariables>(RemoveTrackFromPlaylistDocument, variables)(),
    ...options
  }
    )};

useRemoveTrackFromPlaylistMutation.getKey = () => ['RemoveTrackFromPlaylist'];


useRemoveTrackFromPlaylistMutation.fetcher = (variables: RemoveTrackFromPlaylistMutationVariables, options?: RequestInit['headers']) => fetcher<RemoveTrackFromPlaylistMutation, RemoveTrackFromPlaylistMutationVariables>(RemoveTrackFromPlaylistDocument, variables, options);

export const RenamePlaylistDocument = `
    mutation RenamePlaylist($id: ID!, $name: String!) {
  renamePlaylist(id: $id, name: $name) {
    id
    name
  }
}
    `;

export const useRenamePlaylistMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<RenamePlaylistMutation, TError, RenamePlaylistMutationVariables, TContext>) => {
    
    return useMutation<RenamePlaylistMutation, TError, RenamePlaylistMutationVariables, TContext>(
      {
    mutationKey: ['RenamePlaylist'],
    mutationFn: (variables?: RenamePlaylistMutationVariables) => fetcher<RenamePlaylistMutation, RenamePlaylistMutationVariables>(RenamePlaylistDocument, variables)(),
    ...options
  }
    )};

useRenamePlaylistMutation.getKey = () => ['RenamePlaylist'];


useRenamePlaylistMutation.fetcher = (variables: RenamePlaylistMutationVariables, options?: RequestInit['headers']) => fetcher<RenamePlaylistMutation, RenamePlaylistMutationVariables>(RenamePlaylistDocument, variables, options);

export const CreateFolderDocument = `
    mutation CreateFolder($name: String!) {
  createFolder(name: $name) {
    id
    name
  }
}
    `;

export const useCreateFolderMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<CreateFolderMutation, TError, CreateFolderMutationVariables, TContext>) => {
    
    return useMutation<CreateFolderMutation, TError, CreateFolderMutationVariables, TContext>(
      {
    mutationKey: ['CreateFolder'],
    mutationFn: (variables?: CreateFolderMutationVariables) => fetcher<CreateFolderMutation, CreateFolderMutationVariables>(CreateFolderDocument, variables)(),
    ...options
  }
    )};

useCreateFolderMutation.getKey = () => ['CreateFolder'];


useCreateFolderMutation.fetcher = (variables: CreateFolderMutationVariables, options?: RequestInit['headers']) => fetcher<CreateFolderMutation, CreateFolderMutationVariables>(CreateFolderDocument, variables, options);

export const RenameFolderDocument = `
    mutation RenameFolder($id: ID!, $name: String!) {
  renameFolder(id: $id, name: $name) {
    id
    name
  }
}
    `;

export const useRenameFolderMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<RenameFolderMutation, TError, RenameFolderMutationVariables, TContext>) => {
    
    return useMutation<RenameFolderMutation, TError, RenameFolderMutationVariables, TContext>(
      {
    mutationKey: ['RenameFolder'],
    mutationFn: (variables?: RenameFolderMutationVariables) => fetcher<RenameFolderMutation, RenameFolderMutationVariables>(RenameFolderDocument, variables)(),
    ...options
  }
    )};

useRenameFolderMutation.getKey = () => ['RenameFolder'];


useRenameFolderMutation.fetcher = (variables: RenameFolderMutationVariables, options?: RequestInit['headers']) => fetcher<RenameFolderMutation, RenameFolderMutationVariables>(RenameFolderDocument, variables, options);

export const DeleteFolderDocument = `
    mutation DeleteFolder($id: ID!) {
  deleteFolder(id: $id) {
    id
  }
}
    `;

export const useDeleteFolderMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<DeleteFolderMutation, TError, DeleteFolderMutationVariables, TContext>) => {
    
    return useMutation<DeleteFolderMutation, TError, DeleteFolderMutationVariables, TContext>(
      {
    mutationKey: ['DeleteFolder'],
    mutationFn: (variables?: DeleteFolderMutationVariables) => fetcher<DeleteFolderMutation, DeleteFolderMutationVariables>(DeleteFolderDocument, variables)(),
    ...options
  }
    )};

useDeleteFolderMutation.getKey = () => ['DeleteFolder'];


useDeleteFolderMutation.fetcher = (variables: DeleteFolderMutationVariables, options?: RequestInit['headers']) => fetcher<DeleteFolderMutation, DeleteFolderMutationVariables>(DeleteFolderDocument, variables, options);

export const MovePlaylistToFolderDocument = `
    mutation MovePlaylistToFolder($playlistId: ID!, $folderId: ID!) {
  movePlaylistToFolder(id: $playlistId, folderId: $folderId) {
    ...FolderFragment
  }
}
    ${FolderFragmentFragmentDoc}`;

export const useMovePlaylistToFolderMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<MovePlaylistToFolderMutation, TError, MovePlaylistToFolderMutationVariables, TContext>) => {
    
    return useMutation<MovePlaylistToFolderMutation, TError, MovePlaylistToFolderMutationVariables, TContext>(
      {
    mutationKey: ['MovePlaylistToFolder'],
    mutationFn: (variables?: MovePlaylistToFolderMutationVariables) => fetcher<MovePlaylistToFolderMutation, MovePlaylistToFolderMutationVariables>(MovePlaylistToFolderDocument, variables)(),
    ...options
  }
    )};

useMovePlaylistToFolderMutation.getKey = () => ['MovePlaylistToFolder'];


useMovePlaylistToFolderMutation.fetcher = (variables: MovePlaylistToFolderMutationVariables, options?: RequestInit['headers']) => fetcher<MovePlaylistToFolderMutation, MovePlaylistToFolderMutationVariables>(MovePlaylistToFolderDocument, variables, options);

export const MovePlaylistsToFolderDocument = `
    mutation MovePlaylistsToFolder($playlistIds: [ID!]!, $folderId: ID!) {
  movePlaylistsToFolder(ids: $playlistIds, folderId: $folderId) {
    ...FolderFragment
  }
}
    ${FolderFragmentFragmentDoc}`;

export const useMovePlaylistsToFolderMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<MovePlaylistsToFolderMutation, TError, MovePlaylistsToFolderMutationVariables, TContext>) => {
    
    return useMutation<MovePlaylistsToFolderMutation, TError, MovePlaylistsToFolderMutationVariables, TContext>(
      {
    mutationKey: ['MovePlaylistsToFolder'],
    mutationFn: (variables?: MovePlaylistsToFolderMutationVariables) => fetcher<MovePlaylistsToFolderMutation, MovePlaylistsToFolderMutationVariables>(MovePlaylistsToFolderDocument, variables)(),
    ...options
  }
    )};

useMovePlaylistsToFolderMutation.getKey = () => ['MovePlaylistsToFolder'];


useMovePlaylistsToFolderMutation.fetcher = (variables: MovePlaylistsToFolderMutationVariables, options?: RequestInit['headers']) => fetcher<MovePlaylistsToFolderMutation, MovePlaylistsToFolderMutationVariables>(MovePlaylistsToFolderDocument, variables, options);

export const GetPlaylistsDocument = `
    query GetPlaylists {
  playlists {
    id
    name
    description
    trackCount
  }
}
    `;

export const useGetPlaylistsQuery = <
      TData = GetPlaylistsQuery,
      TError = unknown
    >(
      variables?: GetPlaylistsQueryVariables,
      options?: Omit<UseQueryOptions<GetPlaylistsQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetPlaylistsQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetPlaylistsQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['GetPlaylists'] : ['GetPlaylists', variables],
    queryFn: fetcher<GetPlaylistsQuery, GetPlaylistsQueryVariables>(GetPlaylistsDocument, variables),
    ...options
  }
    )};

useGetPlaylistsQuery.getKey = (variables?: GetPlaylistsQueryVariables) => variables === undefined ? ['GetPlaylists'] : ['GetPlaylists', variables];

export const useInfiniteGetPlaylistsQuery = <
      TData = InfiniteData<GetPlaylistsQuery>,
      TError = unknown
    >(
      variables: GetPlaylistsQueryVariables,
      options: Omit<UseInfiniteQueryOptions<GetPlaylistsQuery, TError, TData>, 'queryKey'> & { queryKey?: UseInfiniteQueryOptions<GetPlaylistsQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useInfiniteQuery<GetPlaylistsQuery, TError, TData>(
      (() => {
    const { queryKey: optionsQueryKey, ...restOptions } = options;
    return {
      queryKey: optionsQueryKey ?? variables === undefined ? ['GetPlaylists.infinite'] : ['GetPlaylists.infinite', variables],
      queryFn: (metaData) => fetcher<GetPlaylistsQuery, GetPlaylistsQueryVariables>(GetPlaylistsDocument, {...variables, ...(metaData.pageParam ?? {})})(),
      ...restOptions
    }
  })()
    )};

useInfiniteGetPlaylistsQuery.getKey = (variables?: GetPlaylistsQueryVariables) => variables === undefined ? ['GetPlaylists.infinite'] : ['GetPlaylists.infinite', variables];


useGetPlaylistsQuery.fetcher = (variables?: GetPlaylistsQueryVariables, options?: RequestInit['headers']) => fetcher<GetPlaylistsQuery, GetPlaylistsQueryVariables>(GetPlaylistsDocument, variables, options);

export const GetRecentPlaylistsDocument = `
    query GetRecentPlaylists {
  recentPlaylists {
    id
    name
    description
    trackCount
  }
}
    `;

export const useGetRecentPlaylistsQuery = <
      TData = GetRecentPlaylistsQuery,
      TError = unknown
    >(
      variables?: GetRecentPlaylistsQueryVariables,
      options?: Omit<UseQueryOptions<GetRecentPlaylistsQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetRecentPlaylistsQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetRecentPlaylistsQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['GetRecentPlaylists'] : ['GetRecentPlaylists', variables],
    queryFn: fetcher<GetRecentPlaylistsQuery, GetRecentPlaylistsQueryVariables>(GetRecentPlaylistsDocument, variables),
    ...options
  }
    )};

useGetRecentPlaylistsQuery.getKey = (variables?: GetRecentPlaylistsQueryVariables) => variables === undefined ? ['GetRecentPlaylists'] : ['GetRecentPlaylists', variables];

export const useInfiniteGetRecentPlaylistsQuery = <
      TData = InfiniteData<GetRecentPlaylistsQuery>,
      TError = unknown
    >(
      variables: GetRecentPlaylistsQueryVariables,
      options: Omit<UseInfiniteQueryOptions<GetRecentPlaylistsQuery, TError, TData>, 'queryKey'> & { queryKey?: UseInfiniteQueryOptions<GetRecentPlaylistsQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useInfiniteQuery<GetRecentPlaylistsQuery, TError, TData>(
      (() => {
    const { queryKey: optionsQueryKey, ...restOptions } = options;
    return {
      queryKey: optionsQueryKey ?? variables === undefined ? ['GetRecentPlaylists.infinite'] : ['GetRecentPlaylists.infinite', variables],
      queryFn: (metaData) => fetcher<GetRecentPlaylistsQuery, GetRecentPlaylistsQueryVariables>(GetRecentPlaylistsDocument, {...variables, ...(metaData.pageParam ?? {})})(),
      ...restOptions
    }
  })()
    )};

useInfiniteGetRecentPlaylistsQuery.getKey = (variables?: GetRecentPlaylistsQueryVariables) => variables === undefined ? ['GetRecentPlaylists.infinite'] : ['GetRecentPlaylists.infinite', variables];


useGetRecentPlaylistsQuery.fetcher = (variables?: GetRecentPlaylistsQueryVariables, options?: RequestInit['headers']) => fetcher<GetRecentPlaylistsQuery, GetRecentPlaylistsQueryVariables>(GetRecentPlaylistsDocument, variables, options);

export const GetMainPlaylistsDocument = `
    query GetMainPlaylists {
  mainPlaylists {
    id
    name
    description
    trackCount
  }
}
    `;

export const useGetMainPlaylistsQuery = <
      TData = GetMainPlaylistsQuery,
      TError = unknown
    >(
      variables?: GetMainPlaylistsQueryVariables,
      options?: Omit<UseQueryOptions<GetMainPlaylistsQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetMainPlaylistsQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetMainPlaylistsQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['GetMainPlaylists'] : ['GetMainPlaylists', variables],
    queryFn: fetcher<GetMainPlaylistsQuery, GetMainPlaylistsQueryVariables>(GetMainPlaylistsDocument, variables),
    ...options
  }
    )};

useGetMainPlaylistsQuery.getKey = (variables?: GetMainPlaylistsQueryVariables) => variables === undefined ? ['GetMainPlaylists'] : ['GetMainPlaylists', variables];

export const useInfiniteGetMainPlaylistsQuery = <
      TData = InfiniteData<GetMainPlaylistsQuery>,
      TError = unknown
    >(
      variables: GetMainPlaylistsQueryVariables,
      options: Omit<UseInfiniteQueryOptions<GetMainPlaylistsQuery, TError, TData>, 'queryKey'> & { queryKey?: UseInfiniteQueryOptions<GetMainPlaylistsQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useInfiniteQuery<GetMainPlaylistsQuery, TError, TData>(
      (() => {
    const { queryKey: optionsQueryKey, ...restOptions } = options;
    return {
      queryKey: optionsQueryKey ?? variables === undefined ? ['GetMainPlaylists.infinite'] : ['GetMainPlaylists.infinite', variables],
      queryFn: (metaData) => fetcher<GetMainPlaylistsQuery, GetMainPlaylistsQueryVariables>(GetMainPlaylistsDocument, {...variables, ...(metaData.pageParam ?? {})})(),
      ...restOptions
    }
  })()
    )};

useInfiniteGetMainPlaylistsQuery.getKey = (variables?: GetMainPlaylistsQueryVariables) => variables === undefined ? ['GetMainPlaylists.infinite'] : ['GetMainPlaylists.infinite', variables];


useGetMainPlaylistsQuery.fetcher = (variables?: GetMainPlaylistsQueryVariables, options?: RequestInit['headers']) => fetcher<GetMainPlaylistsQuery, GetMainPlaylistsQueryVariables>(GetMainPlaylistsDocument, variables, options);

export const GetPlaylistDocument = `
    query GetPlaylist($id: ID!) {
  playlist(id: $id) {
    ...PlaylistFragment
  }
}
    ${PlaylistFragmentFragmentDoc}`;

export const useGetPlaylistQuery = <
      TData = GetPlaylistQuery,
      TError = unknown
    >(
      variables: GetPlaylistQueryVariables,
      options?: Omit<UseQueryOptions<GetPlaylistQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetPlaylistQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetPlaylistQuery, TError, TData>(
      {
    queryKey: ['GetPlaylist', variables],
    queryFn: fetcher<GetPlaylistQuery, GetPlaylistQueryVariables>(GetPlaylistDocument, variables),
    ...options
  }
    )};

useGetPlaylistQuery.getKey = (variables: GetPlaylistQueryVariables) => ['GetPlaylist', variables];

export const useInfiniteGetPlaylistQuery = <
      TData = InfiniteData<GetPlaylistQuery>,
      TError = unknown
    >(
      variables: GetPlaylistQueryVariables,
      options: Omit<UseInfiniteQueryOptions<GetPlaylistQuery, TError, TData>, 'queryKey'> & { queryKey?: UseInfiniteQueryOptions<GetPlaylistQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useInfiniteQuery<GetPlaylistQuery, TError, TData>(
      (() => {
    const { queryKey: optionsQueryKey, ...restOptions } = options;
    return {
      queryKey: optionsQueryKey ?? ['GetPlaylist.infinite', variables],
      queryFn: (metaData) => fetcher<GetPlaylistQuery, GetPlaylistQueryVariables>(GetPlaylistDocument, {...variables, ...(metaData.pageParam ?? {})})(),
      ...restOptions
    }
  })()
    )};

useInfiniteGetPlaylistQuery.getKey = (variables: GetPlaylistQueryVariables) => ['GetPlaylist.infinite', variables];


useGetPlaylistQuery.fetcher = (variables: GetPlaylistQueryVariables, options?: RequestInit['headers']) => fetcher<GetPlaylistQuery, GetPlaylistQueryVariables>(GetPlaylistDocument, variables, options);

export const GetFoldersDocument = `
    query GetFolders {
  folders {
    id
    name
  }
}
    `;

export const useGetFoldersQuery = <
      TData = GetFoldersQuery,
      TError = unknown
    >(
      variables?: GetFoldersQueryVariables,
      options?: Omit<UseQueryOptions<GetFoldersQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetFoldersQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetFoldersQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['GetFolders'] : ['GetFolders', variables],
    queryFn: fetcher<GetFoldersQuery, GetFoldersQueryVariables>(GetFoldersDocument, variables),
    ...options
  }
    )};

useGetFoldersQuery.getKey = (variables?: GetFoldersQueryVariables) => variables === undefined ? ['GetFolders'] : ['GetFolders', variables];

export const useInfiniteGetFoldersQuery = <
      TData = InfiniteData<GetFoldersQuery>,
      TError = unknown
    >(
      variables: GetFoldersQueryVariables,
      options: Omit<UseInfiniteQueryOptions<GetFoldersQuery, TError, TData>, 'queryKey'> & { queryKey?: UseInfiniteQueryOptions<GetFoldersQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useInfiniteQuery<GetFoldersQuery, TError, TData>(
      (() => {
    const { queryKey: optionsQueryKey, ...restOptions } = options;
    return {
      queryKey: optionsQueryKey ?? variables === undefined ? ['GetFolders.infinite'] : ['GetFolders.infinite', variables],
      queryFn: (metaData) => fetcher<GetFoldersQuery, GetFoldersQueryVariables>(GetFoldersDocument, {...variables, ...(metaData.pageParam ?? {})})(),
      ...restOptions
    }
  })()
    )};

useInfiniteGetFoldersQuery.getKey = (variables?: GetFoldersQueryVariables) => variables === undefined ? ['GetFolders.infinite'] : ['GetFolders.infinite', variables];


useGetFoldersQuery.fetcher = (variables?: GetFoldersQueryVariables, options?: RequestInit['headers']) => fetcher<GetFoldersQuery, GetFoldersQueryVariables>(GetFoldersDocument, variables, options);

export const GetFolderDocument = `
    query GetFolder($id: ID!) {
  folder(id: $id) {
    ...FolderFragment
  }
}
    ${FolderFragmentFragmentDoc}`;

export const useGetFolderQuery = <
      TData = GetFolderQuery,
      TError = unknown
    >(
      variables: GetFolderQueryVariables,
      options?: Omit<UseQueryOptions<GetFolderQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetFolderQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetFolderQuery, TError, TData>(
      {
    queryKey: ['GetFolder', variables],
    queryFn: fetcher<GetFolderQuery, GetFolderQueryVariables>(GetFolderDocument, variables),
    ...options
  }
    )};

useGetFolderQuery.getKey = (variables: GetFolderQueryVariables) => ['GetFolder', variables];

export const useInfiniteGetFolderQuery = <
      TData = InfiniteData<GetFolderQuery>,
      TError = unknown
    >(
      variables: GetFolderQueryVariables,
      options: Omit<UseInfiniteQueryOptions<GetFolderQuery, TError, TData>, 'queryKey'> & { queryKey?: UseInfiniteQueryOptions<GetFolderQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useInfiniteQuery<GetFolderQuery, TError, TData>(
      (() => {
    const { queryKey: optionsQueryKey, ...restOptions } = options;
    return {
      queryKey: optionsQueryKey ?? ['GetFolder.infinite', variables],
      queryFn: (metaData) => fetcher<GetFolderQuery, GetFolderQueryVariables>(GetFolderDocument, {...variables, ...(metaData.pageParam ?? {})})(),
      ...restOptions
    }
  })()
    )};

useInfiniteGetFolderQuery.getKey = (variables: GetFolderQueryVariables) => ['GetFolder.infinite', variables];


useGetFolderQuery.fetcher = (variables: GetFolderQueryVariables, options?: RequestInit['headers']) => fetcher<GetFolderQuery, GetFolderQueryVariables>(GetFolderDocument, variables, options);

export const AddServerDocument = `
    mutation AddServer($input: ServerInput!) {
  addServer(input: $input) {
    ...ServerFragment
  }
}
    ${ServerFragmentFragmentDoc}`;

export const useAddServerMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<AddServerMutation, TError, AddServerMutationVariables, TContext>) => {
    
    return useMutation<AddServerMutation, TError, AddServerMutationVariables, TContext>(
      {
    mutationKey: ['AddServer'],
    mutationFn: (variables?: AddServerMutationVariables) => fetcher<AddServerMutation, AddServerMutationVariables>(AddServerDocument, variables)(),
    ...options
  }
    )};

useAddServerMutation.getKey = () => ['AddServer'];


useAddServerMutation.fetcher = (variables: AddServerMutationVariables, options?: RequestInit['headers']) => fetcher<AddServerMutation, AddServerMutationVariables>(AddServerDocument, variables, options);

export const DeleteServerDocument = `
    mutation DeleteServer($id: ID!) {
  deleteServer(id: $id)
}
    `;

export const useDeleteServerMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<DeleteServerMutation, TError, DeleteServerMutationVariables, TContext>) => {
    
    return useMutation<DeleteServerMutation, TError, DeleteServerMutationVariables, TContext>(
      {
    mutationKey: ['DeleteServer'],
    mutationFn: (variables?: DeleteServerMutationVariables) => fetcher<DeleteServerMutation, DeleteServerMutationVariables>(DeleteServerDocument, variables)(),
    ...options
  }
    )};

useDeleteServerMutation.getKey = () => ['DeleteServer'];


useDeleteServerMutation.fetcher = (variables: DeleteServerMutationVariables, options?: RequestInit['headers']) => fetcher<DeleteServerMutation, DeleteServerMutationVariables>(DeleteServerDocument, variables, options);

export const ConnectToServerDocument = `
    mutation ConnectToServer($id: ID!) {
  connectToServer(id: $id) {
    ...ServerFragment
  }
}
    ${ServerFragmentFragmentDoc}`;

export const useConnectToServerMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<ConnectToServerMutation, TError, ConnectToServerMutationVariables, TContext>) => {
    
    return useMutation<ConnectToServerMutation, TError, ConnectToServerMutationVariables, TContext>(
      {
    mutationKey: ['ConnectToServer'],
    mutationFn: (variables?: ConnectToServerMutationVariables) => fetcher<ConnectToServerMutation, ConnectToServerMutationVariables>(ConnectToServerDocument, variables)(),
    ...options
  }
    )};

useConnectToServerMutation.getKey = () => ['ConnectToServer'];


useConnectToServerMutation.fetcher = (variables: ConnectToServerMutationVariables, options?: RequestInit['headers']) => fetcher<ConnectToServerMutation, ConnectToServerMutationVariables>(ConnectToServerDocument, variables, options);

export const DisconnectFromServerDocument = `
    mutation DisconnectFromServer {
  disconnectFromServer {
    id
    name
  }
}
    `;

export const useDisconnectFromServerMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<DisconnectFromServerMutation, TError, DisconnectFromServerMutationVariables, TContext>) => {
    
    return useMutation<DisconnectFromServerMutation, TError, DisconnectFromServerMutationVariables, TContext>(
      {
    mutationKey: ['DisconnectFromServer'],
    mutationFn: (variables?: DisconnectFromServerMutationVariables) => fetcher<DisconnectFromServerMutation, DisconnectFromServerMutationVariables>(DisconnectFromServerDocument, variables)(),
    ...options
  }
    )};

useDisconnectFromServerMutation.getKey = () => ['DisconnectFromServer'];


useDisconnectFromServerMutation.fetcher = (variables?: DisconnectFromServerMutationVariables, options?: RequestInit['headers']) => fetcher<DisconnectFromServerMutation, DisconnectFromServerMutationVariables>(DisconnectFromServerDocument, variables, options);

export const GetSavedServersDocument = `
    query GetSavedServers {
  savedServers {
    ...ServerFragment
  }
}
    ${ServerFragmentFragmentDoc}`;

export const useGetSavedServersQuery = <
      TData = GetSavedServersQuery,
      TError = unknown
    >(
      variables?: GetSavedServersQueryVariables,
      options?: Omit<UseQueryOptions<GetSavedServersQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetSavedServersQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetSavedServersQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['GetSavedServers'] : ['GetSavedServers', variables],
    queryFn: fetcher<GetSavedServersQuery, GetSavedServersQueryVariables>(GetSavedServersDocument, variables),
    ...options
  }
    )};

useGetSavedServersQuery.getKey = (variables?: GetSavedServersQueryVariables) => variables === undefined ? ['GetSavedServers'] : ['GetSavedServers', variables];

export const useInfiniteGetSavedServersQuery = <
      TData = InfiniteData<GetSavedServersQuery>,
      TError = unknown
    >(
      variables: GetSavedServersQueryVariables,
      options: Omit<UseInfiniteQueryOptions<GetSavedServersQuery, TError, TData>, 'queryKey'> & { queryKey?: UseInfiniteQueryOptions<GetSavedServersQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useInfiniteQuery<GetSavedServersQuery, TError, TData>(
      (() => {
    const { queryKey: optionsQueryKey, ...restOptions } = options;
    return {
      queryKey: optionsQueryKey ?? variables === undefined ? ['GetSavedServers.infinite'] : ['GetSavedServers.infinite', variables],
      queryFn: (metaData) => fetcher<GetSavedServersQuery, GetSavedServersQueryVariables>(GetSavedServersDocument, {...variables, ...(metaData.pageParam ?? {})})(),
      ...restOptions
    }
  })()
    )};

useInfiniteGetSavedServersQuery.getKey = (variables?: GetSavedServersQueryVariables) => variables === undefined ? ['GetSavedServers.infinite'] : ['GetSavedServers.infinite', variables];


useGetSavedServersQuery.fetcher = (variables?: GetSavedServersQueryVariables, options?: RequestInit['headers']) => fetcher<GetSavedServersQuery, GetSavedServersQueryVariables>(GetSavedServersDocument, variables, options);

export const GetSourceKindsDocument = `
    query GetSourceKinds {
  sourceKinds {
    kind
    displayName
    needsCredentials
    defaultPort
    fixedUrl
  }
}
    `;

export const useGetSourceKindsQuery = <
      TData = GetSourceKindsQuery,
      TError = unknown
    >(
      variables?: GetSourceKindsQueryVariables,
      options?: Omit<UseQueryOptions<GetSourceKindsQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetSourceKindsQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetSourceKindsQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['GetSourceKinds'] : ['GetSourceKinds', variables],
    queryFn: fetcher<GetSourceKindsQuery, GetSourceKindsQueryVariables>(GetSourceKindsDocument, variables),
    ...options
  }
    )};

useGetSourceKindsQuery.getKey = (variables?: GetSourceKindsQueryVariables) => variables === undefined ? ['GetSourceKinds'] : ['GetSourceKinds', variables];

export const useInfiniteGetSourceKindsQuery = <
      TData = InfiniteData<GetSourceKindsQuery>,
      TError = unknown
    >(
      variables: GetSourceKindsQueryVariables,
      options: Omit<UseInfiniteQueryOptions<GetSourceKindsQuery, TError, TData>, 'queryKey'> & { queryKey?: UseInfiniteQueryOptions<GetSourceKindsQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useInfiniteQuery<GetSourceKindsQuery, TError, TData>(
      (() => {
    const { queryKey: optionsQueryKey, ...restOptions } = options;
    return {
      queryKey: optionsQueryKey ?? variables === undefined ? ['GetSourceKinds.infinite'] : ['GetSourceKinds.infinite', variables],
      queryFn: (metaData) => fetcher<GetSourceKindsQuery, GetSourceKindsQueryVariables>(GetSourceKindsDocument, {...variables, ...(metaData.pageParam ?? {})})(),
      ...restOptions
    }
  })()
    )};

useInfiniteGetSourceKindsQuery.getKey = (variables?: GetSourceKindsQueryVariables) => variables === undefined ? ['GetSourceKinds.infinite'] : ['GetSourceKinds.infinite', variables];


useGetSourceKindsQuery.fetcher = (variables?: GetSourceKindsQueryVariables, options?: RequestInit['headers']) => fetcher<GetSourceKindsQuery, GetSourceKindsQueryVariables>(GetSourceKindsDocument, variables, options);

export const GetConnectedServerDocument = `
    query GetConnectedServer {
  connectedServer {
    ...ServerFragment
  }
}
    ${ServerFragmentFragmentDoc}`;

export const useGetConnectedServerQuery = <
      TData = GetConnectedServerQuery,
      TError = unknown
    >(
      variables?: GetConnectedServerQueryVariables,
      options?: Omit<UseQueryOptions<GetConnectedServerQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetConnectedServerQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetConnectedServerQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['GetConnectedServer'] : ['GetConnectedServer', variables],
    queryFn: fetcher<GetConnectedServerQuery, GetConnectedServerQueryVariables>(GetConnectedServerDocument, variables),
    ...options
  }
    )};

useGetConnectedServerQuery.getKey = (variables?: GetConnectedServerQueryVariables) => variables === undefined ? ['GetConnectedServer'] : ['GetConnectedServer', variables];

export const useInfiniteGetConnectedServerQuery = <
      TData = InfiniteData<GetConnectedServerQuery>,
      TError = unknown
    >(
      variables: GetConnectedServerQueryVariables,
      options: Omit<UseInfiniteQueryOptions<GetConnectedServerQuery, TError, TData>, 'queryKey'> & { queryKey?: UseInfiniteQueryOptions<GetConnectedServerQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useInfiniteQuery<GetConnectedServerQuery, TError, TData>(
      (() => {
    const { queryKey: optionsQueryKey, ...restOptions } = options;
    return {
      queryKey: optionsQueryKey ?? variables === undefined ? ['GetConnectedServer.infinite'] : ['GetConnectedServer.infinite', variables],
      queryFn: (metaData) => fetcher<GetConnectedServerQuery, GetConnectedServerQueryVariables>(GetConnectedServerDocument, {...variables, ...(metaData.pageParam ?? {})})(),
      ...restOptions
    }
  })()
    )};

useInfiniteGetConnectedServerQuery.getKey = (variables?: GetConnectedServerQueryVariables) => variables === undefined ? ['GetConnectedServer.infinite'] : ['GetConnectedServer.infinite', variables];


useGetConnectedServerQuery.fetcher = (variables?: GetConnectedServerQueryVariables, options?: RequestInit['headers']) => fetcher<GetConnectedServerQuery, GetConnectedServerQueryVariables>(GetConnectedServerDocument, variables, options);

export const ClearTracklistDocument = `
    mutation ClearTracklist {
  clearTracklist
}
    `;

export const useClearTracklistMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<ClearTracklistMutation, TError, ClearTracklistMutationVariables, TContext>) => {
    
    return useMutation<ClearTracklistMutation, TError, ClearTracklistMutationVariables, TContext>(
      {
    mutationKey: ['ClearTracklist'],
    mutationFn: (variables?: ClearTracklistMutationVariables) => fetcher<ClearTracklistMutation, ClearTracklistMutationVariables>(ClearTracklistDocument, variables)(),
    ...options
  }
    )};

useClearTracklistMutation.getKey = () => ['ClearTracklist'];


useClearTracklistMutation.fetcher = (variables?: ClearTracklistMutationVariables, options?: RequestInit['headers']) => fetcher<ClearTracklistMutation, ClearTracklistMutationVariables>(ClearTracklistDocument, variables, options);

export const PlayAlbumDocument = `
    mutation PlayAlbum($albumId: ID!, $position: Int, $shuffle: Boolean!) {
  playAlbum(id: $albumId, position: $position, shuffle: $shuffle)
}
    `;

export const usePlayAlbumMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<PlayAlbumMutation, TError, PlayAlbumMutationVariables, TContext>) => {
    
    return useMutation<PlayAlbumMutation, TError, PlayAlbumMutationVariables, TContext>(
      {
    mutationKey: ['PlayAlbum'],
    mutationFn: (variables?: PlayAlbumMutationVariables) => fetcher<PlayAlbumMutation, PlayAlbumMutationVariables>(PlayAlbumDocument, variables)(),
    ...options
  }
    )};

usePlayAlbumMutation.getKey = () => ['PlayAlbum'];


usePlayAlbumMutation.fetcher = (variables: PlayAlbumMutationVariables, options?: RequestInit['headers']) => fetcher<PlayAlbumMutation, PlayAlbumMutationVariables>(PlayAlbumDocument, variables, options);

export const PlayArtistTracksDocument = `
    mutation PlayArtistTracks($artistId: ID!, $position: Int, $shuffle: Boolean!) {
  playArtistTracks(id: $artistId, position: $position, shuffle: $shuffle)
}
    `;

export const usePlayArtistTracksMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<PlayArtistTracksMutation, TError, PlayArtistTracksMutationVariables, TContext>) => {
    
    return useMutation<PlayArtistTracksMutation, TError, PlayArtistTracksMutationVariables, TContext>(
      {
    mutationKey: ['PlayArtistTracks'],
    mutationFn: (variables?: PlayArtistTracksMutationVariables) => fetcher<PlayArtistTracksMutation, PlayArtistTracksMutationVariables>(PlayArtistTracksDocument, variables)(),
    ...options
  }
    )};

usePlayArtistTracksMutation.getKey = () => ['PlayArtistTracks'];


usePlayArtistTracksMutation.fetcher = (variables: PlayArtistTracksMutationVariables, options?: RequestInit['headers']) => fetcher<PlayArtistTracksMutation, PlayArtistTracksMutationVariables>(PlayArtistTracksDocument, variables, options);

export const PlayPlaylistDocument = `
    mutation PlayPlaylist($playlistId: ID!, $position: Int, $shuffle: Boolean!) {
  playPlaylist(id: $playlistId, position: $position, shuffle: $shuffle)
}
    `;

export const usePlayPlaylistMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<PlayPlaylistMutation, TError, PlayPlaylistMutationVariables, TContext>) => {
    
    return useMutation<PlayPlaylistMutation, TError, PlayPlaylistMutationVariables, TContext>(
      {
    mutationKey: ['PlayPlaylist'],
    mutationFn: (variables?: PlayPlaylistMutationVariables) => fetcher<PlayPlaylistMutation, PlayPlaylistMutationVariables>(PlayPlaylistDocument, variables)(),
    ...options
  }
    )};

usePlayPlaylistMutation.getKey = () => ['PlayPlaylist'];


usePlayPlaylistMutation.fetcher = (variables: PlayPlaylistMutationVariables, options?: RequestInit['headers']) => fetcher<PlayPlaylistMutation, PlayPlaylistMutationVariables>(PlayPlaylistDocument, variables, options);

export const PlayTrackAtDocument = `
    mutation PlayTrackAt($position: Int!) {
  playTrackAt(position: $position)
}
    `;

export const usePlayTrackAtMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<PlayTrackAtMutation, TError, PlayTrackAtMutationVariables, TContext>) => {
    
    return useMutation<PlayTrackAtMutation, TError, PlayTrackAtMutationVariables, TContext>(
      {
    mutationKey: ['PlayTrackAt'],
    mutationFn: (variables?: PlayTrackAtMutationVariables) => fetcher<PlayTrackAtMutation, PlayTrackAtMutationVariables>(PlayTrackAtDocument, variables)(),
    ...options
  }
    )};

usePlayTrackAtMutation.getKey = () => ['PlayTrackAt'];


usePlayTrackAtMutation.fetcher = (variables: PlayTrackAtMutationVariables, options?: RequestInit['headers']) => fetcher<PlayTrackAtMutation, PlayTrackAtMutationVariables>(PlayTrackAtDocument, variables, options);

export const RemoveTrackAtDocument = `
    mutation RemoveTrackAt($position: Int!) {
  removeTrack(position: $position)
}
    `;

export const useRemoveTrackAtMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<RemoveTrackAtMutation, TError, RemoveTrackAtMutationVariables, TContext>) => {
    
    return useMutation<RemoveTrackAtMutation, TError, RemoveTrackAtMutationVariables, TContext>(
      {
    mutationKey: ['RemoveTrackAt'],
    mutationFn: (variables?: RemoveTrackAtMutationVariables) => fetcher<RemoveTrackAtMutation, RemoveTrackAtMutationVariables>(RemoveTrackAtDocument, variables)(),
    ...options
  }
    )};

useRemoveTrackAtMutation.getKey = () => ['RemoveTrackAt'];


useRemoveTrackAtMutation.fetcher = (variables: RemoveTrackAtMutationVariables, options?: RequestInit['headers']) => fetcher<RemoveTrackAtMutation, RemoveTrackAtMutationVariables>(RemoveTrackAtDocument, variables, options);

export const PlayNextDocument = `
    mutation PlayNext($trackId: ID!) {
  playNext(id: $trackId)
}
    `;

export const usePlayNextMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<PlayNextMutation, TError, PlayNextMutationVariables, TContext>) => {
    
    return useMutation<PlayNextMutation, TError, PlayNextMutationVariables, TContext>(
      {
    mutationKey: ['PlayNext'],
    mutationFn: (variables?: PlayNextMutationVariables) => fetcher<PlayNextMutation, PlayNextMutationVariables>(PlayNextDocument, variables)(),
    ...options
  }
    )};

usePlayNextMutation.getKey = () => ['PlayNext'];


usePlayNextMutation.fetcher = (variables: PlayNextMutationVariables, options?: RequestInit['headers']) => fetcher<PlayNextMutation, PlayNextMutationVariables>(PlayNextDocument, variables, options);

export const AddTracksDocument = `
    mutation AddTracks($tracks: [TrackInput!]!) {
  addTracks(tracks: $tracks)
}
    `;

export const useAddTracksMutation = <
      TError = unknown,
      TContext = unknown
    >(options?: UseMutationOptions<AddTracksMutation, TError, AddTracksMutationVariables, TContext>) => {
    
    return useMutation<AddTracksMutation, TError, AddTracksMutationVariables, TContext>(
      {
    mutationKey: ['AddTracks'],
    mutationFn: (variables?: AddTracksMutationVariables) => fetcher<AddTracksMutation, AddTracksMutationVariables>(AddTracksDocument, variables)(),
    ...options
  }
    )};

useAddTracksMutation.getKey = () => ['AddTracks'];


useAddTracksMutation.fetcher = (variables: AddTracksMutationVariables, options?: RequestInit['headers']) => fetcher<AddTracksMutation, AddTracksMutationVariables>(AddTracksDocument, variables, options);

export const GetTracklistDocument = `
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

export const useGetTracklistQuery = <
      TData = GetTracklistQuery,
      TError = unknown
    >(
      variables?: GetTracklistQueryVariables,
      options?: Omit<UseQueryOptions<GetTracklistQuery, TError, TData>, 'queryKey'> & { queryKey?: UseQueryOptions<GetTracklistQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useQuery<GetTracklistQuery, TError, TData>(
      {
    queryKey: variables === undefined ? ['GetTracklist'] : ['GetTracklist', variables],
    queryFn: fetcher<GetTracklistQuery, GetTracklistQueryVariables>(GetTracklistDocument, variables),
    ...options
  }
    )};

useGetTracklistQuery.getKey = (variables?: GetTracklistQueryVariables) => variables === undefined ? ['GetTracklist'] : ['GetTracklist', variables];

export const useInfiniteGetTracklistQuery = <
      TData = InfiniteData<GetTracklistQuery>,
      TError = unknown
    >(
      variables: GetTracklistQueryVariables,
      options: Omit<UseInfiniteQueryOptions<GetTracklistQuery, TError, TData>, 'queryKey'> & { queryKey?: UseInfiniteQueryOptions<GetTracklistQuery, TError, TData>['queryKey'] }
    ) => {
    
    return useInfiniteQuery<GetTracklistQuery, TError, TData>(
      (() => {
    const { queryKey: optionsQueryKey, ...restOptions } = options;
    return {
      queryKey: optionsQueryKey ?? variables === undefined ? ['GetTracklist.infinite'] : ['GetTracklist.infinite', variables],
      queryFn: (metaData) => fetcher<GetTracklistQuery, GetTracklistQueryVariables>(GetTracklistDocument, {...variables, ...(metaData.pageParam ?? {})})(),
      ...restOptions
    }
  })()
    )};

useInfiniteGetTracklistQuery.getKey = (variables?: GetTracklistQueryVariables) => variables === undefined ? ['GetTracklist.infinite'] : ['GetTracklist.infinite', variables];


useGetTracklistQuery.fetcher = (variables?: GetTracklistQueryVariables, options?: RequestInit['headers']) => fetcher<GetTracklistQuery, GetTracklistQueryVariables>(GetTracklistDocument, variables, options);
