import { useMutation, useQuery } from "@tanstack/react-query";
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
import { paletteOpenAtom } from "../../State";
import { Icons, Toggle } from "../UI";
import {
  gql,
  STATION_FIELDS,
  type Station,
} from "../../Containers/Radio/api";
import CommandPalette, { type PaletteEntry } from "./CommandPalette";

/** How many of each kind the list shows, so no one kind fills it. */
const PER_KIND = 5;

/** Builds the palette's entries from the library, playlists and extensions. */
const CommandPaletteWithData = () => {
  const [open, setOpen] = useAtom(paletteOpenAtom);
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
    if (!needle) return [];

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
        run: () => playTrack(track.id),
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

    // Servers and cast targets: the sidebar's status row is the only other way
    // to reach them, and it is not where anyone looks for one by name.
    const servers: PaletteEntry[] = [
      ...castDevices.map((device) => ({ device, cast: true })),
      ...devices.map((device) => ({ device, cast: false })),
    ]
      .filter(({ device }) => matches(device.name) || matches(device.id))
      .slice(0, PER_KIND)
      .map(({ device, cast }) => {
        const active = cast
          ? currentCastDevice?.id === device.id
          : currentDevice?.id === device.id;
        return {
          key: `server-${cast ? "cast" : "mp"}-${device.id}`,
          kind: "server" as const,
          title: device.name,
          subtitle: active
            ? cast
              ? "Cast · playing here"
              : "music-player · playing here"
            : cast
              ? "Cast"
              : "music-player",
          icon: Icons.device,
          run: () => {
            if (active) return;
            if (cast) connectToCastDevice({ id: device.id });
            else connectToDevice({ id: device.id });
          },
        };
      });

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
