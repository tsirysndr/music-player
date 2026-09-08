import { graphql, HttpResponse } from "msw";
import * as fixtures from "./fixtures";

/**
 * The default GraphQL responses.
 *
 * The daemon has one endpoint and the codegen'd hooks name every operation, so
 * MSW can route on the operation name — a test that wants a different answer
 * calls `server.use(graphql.query("GetAlbums", …))` and leaves the rest alone.
 *
 * The bodies come from `fixtures.ts`, which is generated from a real library
 * rather than written by hand.
 */

export const emptyTracklist = {
  tracklistTracks: {
    __typename: "Tracklist",
    nextTracks: [],
    previousTracks: [],
  },
};

export const handlers = [
  graphql.query("GetAlbums", () =>
    HttpResponse.json({ data: { albums: fixtures.albums } })
  ),
  graphql.query("GetArtists", () =>
    HttpResponse.json({ data: { artists: fixtures.artists } })
  ),
  graphql.query("GetTracks", () =>
    HttpResponse.json({ data: { tracks: fixtures.tracks } })
  ),
  graphql.query("GetAlbum", () =>
    HttpResponse.json({ data: { album: fixtures.album } })
  ),
  graphql.query("GetArtist", () =>
    HttpResponse.json({ data: { artist: fixtures.artistDetail } })
  ),
  graphql.query("GetExtensions", () =>
    HttpResponse.json({ data: { extensions: extensions } })
  ),
  graphql.query("GetPlaylists", () =>
    HttpResponse.json({ data: { playlists } })
  ),
  graphql.query("GetRecentPlaylists", () =>
    HttpResponse.json({ data: { recentPlaylists: playlists } })
  ),
  graphql.query("GetMainPlaylists", () =>
    HttpResponse.json({ data: { mainPlaylists: playlists } })
  ),
  graphql.query("GetFolders", () => HttpResponse.json({ data: { folders } })),
  graphql.query("GetTracklist", () =>
    HttpResponse.json({ data: emptyTracklist })
  ),
  graphql.query("CurrentlyPlayingSong", () =>
    HttpResponse.json({ data: { currentlyPlayingSong: null } })
  ),
  graphql.query("ListDevices", () =>
    HttpResponse.json({ data: { listDevices: [] } })
  ),
  graphql.query("ListCastDevices", () =>
    HttpResponse.json({ data: { listCastDevices: [] } })
  ),
  graphql.query("ConnectedDevice", () =>
    HttpResponse.json({ data: { connectedDevice: null } })
  ),
  graphql.query("ConnectedCastDevice", () =>
    HttpResponse.json({ data: { connectedCastDevice: null } })
  ),
  graphql.query("Search", () =>
    HttpResponse.json({
      data: {
        search: {
          __typename: "SearchResult",
          albums: [],
          artists: [],
          tracks: [],
        },
      },
    })
  ),
  // Mutations answer truthily so an optimistic UI is not rolled back mid-test.
  graphql.mutation("Play", () => HttpResponse.json({ data: { play: true } })),
  graphql.mutation("Pause", () => HttpResponse.json({ data: { pause: true } })),
  graphql.mutation("Next", () => HttpResponse.json({ data: { next: true } })),
  graphql.mutation("Previous", () =>
    HttpResponse.json({ data: { previous: true } })
  ),
  graphql.mutation("Seek", () => HttpResponse.json({ data: { seek: true } })),
  graphql.mutation("PlayNext", () =>
    HttpResponse.json({ data: { playNext: true } })
  ),
  graphql.mutation("PlayAlbum", () =>
    HttpResponse.json({ data: { playAlbum: true } })
  ),
  graphql.mutation("PlayArtistTracks", () =>
    HttpResponse.json({ data: { playArtistTracks: true } })
  ),
  graphql.mutation("PlayTrackAt", () =>
    HttpResponse.json({ data: { playTrackAt: true } })
  ),
  graphql.mutation("LikeTrack", () =>
    HttpResponse.json({ data: { likeTrack: true } })
  ),
  graphql.query("GetSavedServers", () =>
    HttpResponse.json({ data: { savedServers } })
  ),
  graphql.query("GetSourceKinds", () =>
    HttpResponse.json({ data: { sourceKinds } })
  ),
  graphql.query("GetConnectedServer", () =>
    HttpResponse.json({ data: { connectedServer: null } })
  ),
  // Anything the app asks for that has no operation name (the hand-written
  // `fetcher` calls in the radio and liked pages) falls through to here.
  graphql.operation(() => HttpResponse.json({ data: {} })),
];

export const playlists = [
  {
    __typename: "Playlist",
    id: "pl-late-night",
    name: "Late night",
    description: "For the small hours",
  },
  {
    __typename: "Playlist",
    id: "pl-focus",
    name: "Focus",
    description: null,
  },
];

export const folders = [
  { __typename: "Folder", id: "folder-moods", name: "Moods" },
];

export const extensions = [
  {
    __typename: "Extension",
    id: "fm.atradio.lyrics-provider",
    name: "Lyrics Provider",
    version: "1.2.0",
    author: "atradio",
    description: "Fetches lyrics for the track that is playing.",
    homepage: "https://example.com",
    repository: "",
    license: "MIT",
    logo: "",
    topics: ["lyrics"],
    capabilities: ["metadata"],
    allowedHosts: ["api.lyrics.example.com"],
    libraryRead: false,
    status: "enabled",
    path: "/extensions/lyrics-provider",
  },
  {
    __typename: "Extension",
    id: "com.example.mood",
    name: "Mood Predicate",
    version: "0.1.0",
    author: "Someone",
    description: "Adds ext:mood to smart-playlist filters.",
    homepage: "",
    repository: "",
    license: "MIT",
    logo: "",
    topics: ["smart-playlists"],
    capabilities: ["predicates"],
    allowedHosts: [],
    libraryRead: true,
    status: "disabled",
    path: "/extensions/mood-predicate",
  },
];

/** As the daemon's provider registry describes itself. */
export const sourceKinds = [
  {
    __typename: "SourceKind",
    kind: "subsonic",
    displayName: "Subsonic / Navidrome",
    needsCredentials: true,
    defaultPort: 4533,
  },
  {
    __typename: "SourceKind",
    kind: "jellyfin",
    displayName: "Jellyfin",
    needsCredentials: true,
    defaultPort: 8096,
  },
  {
    __typename: "SourceKind",
    kind: "music-player",
    displayName: "music-player",
    needsCredentials: false,
    defaultPort: 5053,
  },
];

export const savedServers = [
  {
    __typename: "Server",
    id: "5f2b1c9d4e7a8b3c6d0e1f2a3b4c5d6e",
    kind: "subsonic",
    name: "Living room NAS",
    url: "http://192.168.1.10:4533",
    username: "tsiry",
    hasPassword: true,
    connected: false,
  },
  {
    __typename: "Server",
    id: "9a8b7c6d5e4f3a2b1c0d9e8f7a6b5c4d",
    kind: "jellyfin",
    name: "Media",
    url: "http://media.home.lan:8096",
    username: "tsiry",
    hasPassword: true,
    connected: false,
  },
];
