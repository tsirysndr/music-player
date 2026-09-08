import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useAtom } from "jotai";
import { useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import {
  useGetExtensionsQuery,
  useGetPlaylistsQuery,
  useSearchQuery,
  useSetExtensionEnabledMutation,
} from "../../Hooks/GraphQL";
import { useDevices } from "../../Hooks/useDevices";
import { usePlayTrack } from "../../Hooks/usePlayTrack";
import { paletteOpenAtom, serverSwitcherOpenAtom } from "../../State";
import {
  useAddServerMutation,
  useAddTrackToPlaylistMutation,
  usePlayAlbumMutation,
  useConnectToServerMutation,
  useDisconnectFromServerMutation,
  useGetSavedServersQuery,
} from "../../Hooks/GraphQL";
import {
  ContextMenuItem,
  ContextMenuLabel,
  ContextMenuSeparator,
  Icons,
  Toggle,
  type IconComponent,
} from "../UI";
import {
  gql,
  STATION_FIELDS,
  type Station,
} from "../../Containers/Radio/api";
import CommandPalette, { type PaletteEntry } from "./CommandPalette";

/** How many of each kind the list shows, so no one kind fills it. */
const PER_KIND = 5;

/** What a server switch invalidates. */
const LIBRARY_QUERIES = [
  "GetTracks",
  "GetAlbums",
  "GetArtists",
  "GetAlbum",
  "GetArtist",
  "GetPlaylists",
  "GetPlaylist",
  "GetLikedTracks",
  "Search",
  "GetSavedServers",
];

/** How each kind of server shows itself. */
const KIND_ICON: Record<string, IconComponent> = {
  jellyfin: Icons.jellyfin,
  subsonic: Icons.navidrome,
  navidrome: Icons.navidrome,
  "music-player": Icons.server,
  kodi: Icons.device,
  plex: Icons.disc,
};

/** Builds the palette's entries from the library, playlists and extensions. */
const CommandPaletteWithData = () => {
  const [paletteOpen, setPaletteOpen] = useAtom(paletteOpenAtom);
  const [switcherOpen, setSwitcherOpen] = useAtom(serverSwitcherOpenAtom);
  // One overlay serves both: the switcher is the palette scoped to servers.
  const scope = switcherOpen ? ("servers" as const) : undefined;
  const queryClient = useQueryClient();
  // Not gated to the switcher any more: search is federated, so every result
  // list needs to know which library is connected in order to label its rows
  // and offer the actions that work on them.
  const { data: serverData } = useGetSavedServersQuery();
  const connectedServer = (serverData?.savedServers ?? []).find(
    (server) => server.connected
  );
  const addTrackToPlaylist = useAddTrackToPlaylistMutation().mutate;
  const playAlbum = usePlayAlbumMutation().mutate;

  /**
   * A result's library, for the row's label. `null` from the daemon means this
   * machine; it is only worth saying when there is another library to confuse
   * it with.
   */
  const originLabel = (source?: string | null) =>
    connectedServer ? (source ?? "this machine") : undefined;

  /**
   * The playlists a track from `source` could join.
   *
   * A playlist belongs to one library. With a server connected the playlists
   * on offer are its own, so a local result has none it can join — and saying
   * so by omission beats a row that saves and then will not play.
   */
  const playlistsFor = (source?: string | null) => {
    const remote = !!source;
    if (!!connectedServer !== remote) {
      return [];
    }
    return playlistData?.playlists ?? [];
  };
  const addServer = useAddServerMutation();
  const connect = useConnectToServerMutation();
  const disconnect = useDisconnectFromServerMutation();

  /** Everything on screen came from the old server, so it all goes. */
  const refreshLibrary = () =>
    queryClient.invalidateQueries({
      predicate: (entry: { queryKey: readonly unknown[] }) =>
        LIBRARY_QUERIES.includes(String(entry.queryKey[0])),
    });

  const connectToServer = async ({ id }: { id: string }) => {
    await connect.mutateAsync({ id });
    await refreshLibrary();
  };

  const disconnectFromServer = async (_: Record<string, never>) => {
    await disconnect.mutateAsync({});
    await refreshLibrary();
  };

  /**
   * Save whatever was typed as a peer daemon and switch to it. Bare hosts get
   * the daemon's own port and scheme, so `studio.lan` is enough.
   */
  const connectToTyped = async (typed: string) => {
    const url = /^https?:\/\//.test(typed) ? typed : `http://${typed}`;
    const withPort = /:\d+/.test(url.replace(/^https?:\/\//, ""))
      ? url
      : `${url}:5053`;
    const added = await addServer.mutateAsync({
      input: { kind: "music-player", name: typed, url: withPort },
    });
    await connectToServer({ id: added.addServer.id });
  };
  const open = paletteOpen || switcherOpen;
  const setOpen = (next: boolean) => {
    if (!next) {
      setPaletteOpen(false);
      setSwitcherOpen(false);
    } else if (!switcherOpen) {
      setPaletteOpen(true);
    }
  };
  const [query, setQuery] = useState("");
  const [selected, setSelected] = useState(0);
  const navigate = useNavigate();
  const playTrack = usePlayTrack();
  const {
    devices,
    castDevices,
    currentDevice,
    currentCastDevice,
    connectToDevice,
    connectToCastDevice,
  } = useDevices();

  const needle = query.trim().toLowerCase();

  // Only ask the daemon once there is something to search for; an empty
  // palette would otherwise fetch the whole library every time it opens.
  const { data: search, isFetching } = useSearchQuery(
    { keyword: query.trim() },
    { enabled: open && needle.length > 0 }
  );
  const { data: playlistData } = useGetPlaylistsQuery(undefined, {
    enabled: open,
  });
  const { data: extensionData, refetch: refetchExtensions } =
    useGetExtensionsQuery({}, { enabled: open });

  // Radio search goes over the network to radio-browser, so unlike playlists
  // and extensions it is not matched client-side — and it waits for two
  // characters rather than firing on the first keystroke.
  const { data: radios, isFetching: radiosFetching } = useQuery({
    queryKey: ["palette-radios", needle],
    queryFn: async () => {
      const data = await gql<{ radios: Station[] }>(
        `query($query:String){ radios(query:$query){${STATION_FIELDS}}}`,
        { query: query.trim() }
      );
      return data.radios ?? [];
    },
    enabled: open && needle.length > 1,
    staleTime: 60_000,
  });

  const playRadio = useMutation({
    mutationFn: (station: Station) =>
      gql(`mutation($station:RadioStationInput!){playRadio(station:$station)}`, {
        station,
      }),
  });
  // A new identity every render, so the memo below takes the stable part.
  const startStation = playRadio.mutate;

  const setExtensionEnabled = useSetExtensionEnabledMutation({
    onSettled: () => refetchExtensions(),
  });
  // The mutation object is a new identity on every render, so the memo below
  // takes the one stable thing off it rather than the object itself.
  const toggleExtension = setExtensionEnabled.mutate;

  const entries = useMemo<PaletteEntry[]>(() => {
    // The library palette waits for a query — searching it costs a round trip.
    // The switcher does not: its list is already loaded, and listing it is
    // what makes it a switcher.
    if (!needle && scope !== "servers") return [];

    const matches = (value?: string | null) =>
      !!value && value.toLowerCase().includes(needle);

    const tracks: PaletteEntry[] = (search?.search.tracks ?? [])
      .slice(0, PER_KIND)
      .map((track) => ({
        key: `track-${track.id}`,
        kind: "track",
        title: track.title,
        subtitle: track.artist,
        cover: track.cover ? `/covers/${track.cover}` : undefined,
        icon: Icons.music,
        origin: originLabel(track.source),
        run: () => playTrack(track.id),
        menu: (
          <>
            <ContextMenuItem
              icon={Icons.play}
              label="Play"
              onClick={() => playTrack(track.id)}
            />
            {/* A playlist lives in one library, so a track can only join a
                playlist from the same one. Offering it otherwise produces a
                row that saves and then cannot be played. */}
            {playlistsFor(track.source).length > 0 && (
              <>
                <ContextMenuSeparator />
                <ContextMenuLabel>Add to playlist</ContextMenuLabel>
                {playlistsFor(track.source).map((playlist) => (
                  <ContextMenuItem
                    key={playlist.id}
                    icon={Icons.listMusic}
                    label={playlist.name}
                    onClick={() =>
                      addTrackToPlaylist({
                        playlistId: playlist.id,
                        trackId: track.id,
                      })
                    }
                  />
                ))}
              </>
            )}
            {track.albumId && (
              <>
                <ContextMenuSeparator />
                <ContextMenuItem
                  icon={Icons.disc}
                  label="Go to album"
                  onClick={() => navigate(`/albums/${track.albumId}`)}
                />
              </>
            )}
            {track.artistId && (
              <ContextMenuItem
                icon={Icons.artist}
                label="Go to artist"
                onClick={() => navigate(`/artists/${track.artistId}`)}
              />
            )}
          </>
        ),
      }));

    // Albums and artists go to their page rather than playing: opening one is
    // what a search result is for, and playing is one click away once there.
    const albums: PaletteEntry[] = (search?.search.albums ?? [])
      .slice(0, PER_KIND)
      .map((album) => ({
        key: `album-${album.id}`,
        kind: "album",
        title: album.title,
        subtitle: album.artist,
        cover: album.cover ? `/covers/${album.cover}` : undefined,
        icon: Icons.disc,
        run: () => navigate(`/albums/${album.id}`),
        origin: originLabel(album.source),
        menu: (
          <>
            <ContextMenuItem
              icon={Icons.play}
              label="Play"
              onClick={() => playAlbum({ albumId: album.id, shuffle: false })}
            />
            <ContextMenuItem
              icon={Icons.shuffle}
              label="Shuffle"
              onClick={() => playAlbum({ albumId: album.id, shuffle: true })}
            />
            <ContextMenuSeparator />
            <ContextMenuItem
              icon={Icons.disc}
              label="Go to album"
              onClick={() => navigate(`/albums/${album.id}`)}
            />
          </>
        ),
      }));

    const artists: PaletteEntry[] = (search?.search.artists ?? [])
      .slice(0, PER_KIND)
      .map((artist) => ({
        key: `artist-${artist.id}`,
        kind: "artist",
        title: artist.name,
        cover: artist.picture,
        icon: Icons.artist,
        run: () => navigate(`/artists/${artist.id}`),
        origin: originLabel(artist.source),
      }));

    // Matched here rather than on the daemon: both lists are short and already
    // cached, so a round trip would only add latency to a keystroke.
    const playlists: PaletteEntry[] = (playlistData?.playlists ?? [])
      .filter(
        (playlist) => matches(playlist.name) || matches(playlist.description)
      )
      .slice(0, PER_KIND)
      .map((playlist) => ({
        key: `playlist-${playlist.id}`,
        kind: "playlist",
        title: playlist.name,
        subtitle: playlist.description ?? undefined,
        icon: Icons.playlist,
        run: () => navigate(`/playlists/${playlist.id}`),
      }));

    const extensions: PaletteEntry[] = (extensionData?.extensions ?? [])
      .filter(
        (extension) =>
          matches(extension.name) ||
          matches(extension.id) ||
          matches(extension.description) ||
          extension.topics.some(matches) ||
          extension.capabilities.some(matches)
      )
      .slice(0, PER_KIND)
      .map((extension) => ({
        key: `extension-${extension.id}`,
        kind: "extension",
        title: extension.name,
        subtitle: extension.description || extension.id,
        icon: Icons.extension,
        run: () => navigate("/extensions"),
        // Switching one is the thing you came here to do, so it is on the row
        // rather than a trip to the extensions page.
        action: (
          <Toggle
            checked={extension.status === "enabled"}
            label={`Enable ${extension.name}`}
            onChange={(next) =>
              toggleExtension({ id: extension.id, enabled: next })
            }
          />
        ),
      }));

    // The servers the library can be read *from*. Where the audio comes out
    // is the Play to dialog — a different question, and it used to be this
    // list, which is why picking a "server" here used to start casting.
    const savedServers: PaletteEntry[] = (serverData?.savedServers ?? [])
      .filter((server) => matches(server.name) || matches(server.url))
      .slice(0, scope === "servers" ? 50 : PER_KIND)
      .map((server) => ({
        key: `server-${server.id}`,
        kind: "server" as const,
        title: server.name,
        subtitle: server.connected
          ? `${server.url} · connected`
          : server.url,
        icon: KIND_ICON[server.kind] ?? Icons.server,
        run: () => {
          if (server.connected) return;
          connectToServer({ id: server.id });
        },
      }));

    // The local library is always an option, and is where it starts.
    const local: PaletteEntry[] =
      scope === "servers" && (!needle || matches("this machine"))
        ? [
            {
              key: "server-local",
              kind: "server" as const,
              title: "This machine",
              subtitle: connectedServer
                ? "the daemon's own library"
                : "the daemon's own library · connected",
              icon: Icons.server,
              run: () => {
                if (connectedServer) disconnectFromServer({});
              },
            },
          ]
        : [];

    const servers = [...local, ...savedServers];

    const stations: PaletteEntry[] = (radios ?? [])
      .slice(0, PER_KIND)
      .map((station) => ({
        key: `radio-${station.id}`,
        kind: "radio" as const,
        title: station.name,
        subtitle:
          [station.genre, station.country, station.bitrate ? `${station.bitrate}kbps` : ""]
            .filter(Boolean)
            .join(" · ") || station.source,
        cover: station.logo || undefined,
        icon: Icons.broadcast,
        // A station has nowhere to navigate to, so activating one plays it —
        // the same rule the radio page's rows follow.
        run: () => startStation(station),
      }));

    // Nothing matched, but the user typed something: offer to connect to it.
    // A peer daemon is the one kind that needs no credentials, so it is the
    // only thing that can be connected to from a bare address.
    const suggestion: PaletteEntry[] =
      scope === "servers" && servers.length === 0 && needle.length > 0
        ? [
            {
              key: "server-connect-typed",
              kind: "server" as const,
              title: `Connect to ${query.trim()}`,
              subtitle: "music-player · adds it and switches to it",
              icon: Icons.connect,
              run: () => connectToTyped(query.trim()),
            },
          ]
        : [];

    if (scope === "servers") {
      return [...servers, ...suggestion];
    }

    return [
      ...tracks,
      ...albums,
      ...artists,
      ...playlists,
      ...extensions,
      ...servers,
      ...stations,
    ];
  }, [
    needle,
    scope,
    query,
    serverData,
    connectedServer,
    search,
    playlistData,
    extensionData,
    devices,
    castDevices,
    currentDevice,
    currentCastDevice,
    connectToDevice,
    connectToCastDevice,
    radios,
    startStation,
    navigate,
    playTrack,
    toggleExtension,
  ]);

  // A new result set invalidates whatever was highlighted.
  useEffect(() => setSelected(0), [entries.length]);

  const close = () => {
    setOpen(false);
    setQuery("");
    setSelected(0);
  };

  return (
    <CommandPalette
      open={open}
      query={query}
      entries={entries}
      selected={selected}
      loading={isFetching || radiosFetching}
      onQueryChange={(next) => {
        setQuery(next);
        setSelected(0);
      }}
      scope={scope}
      onAddServer={() => {
        close();
        navigate("/servers?add=1");
      }}
      hint={
        scope === "servers" ? (
          <>
            <span>enter connect</span>
            <span>C-n add</span>
            <span>esc close</span>
          </>
        ) : undefined
      }
      onSelect={setSelected}
      onActivate={(entry) => {
        entry.run();
        close();
      }}
      onClose={close}
    />
  );
};

export default CommandPaletteWithData;
