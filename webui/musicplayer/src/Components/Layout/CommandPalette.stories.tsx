import type { Meta, StoryObj } from "@storybook/react-vite";
import { noop } from "../../Stories/fixtures";
import { Icons, Toggle } from "../UI";
import CommandPalette, { type PaletteEntry } from "./CommandPalette";

const TIDAL = "https://resources.tidal.com/images";

const track: PaletteEntry = {
  key: "track-1",
  kind: "track",
  title: "Around the World",
  subtitle: "Daft Punk",
  cover: `${TIDAL}/f87d9afc/075e/43f4/bbbc/7770b46cb8aa/320x320.jpg`,
  icon: Icons.music,
  run: noop,
};

const album: PaletteEntry = {
  key: "album-1",
  kind: "album",
  title: "Discovery",
  subtitle: "Daft Punk",
  cover: `${TIDAL}/f87d9afc/075e/43f4/bbbc/7770b46cb8aa/320x320.jpg`,
  icon: Icons.disc,
  run: noop,
};

// No picture, so the round artist placeholder is exercised too.
const artist: PaletteEntry = {
  key: "artist-1",
  kind: "artist",
  title: "Daft Punk",
  icon: Icons.artist,
  run: noop,
};

const playlist: PaletteEntry = {
  key: "playlist-1",
  kind: "playlist",
  title: "Late night",
  subtitle: "For the small hours",
  icon: Icons.playlist,
  run: noop,
};

const extension: PaletteEntry = {
  key: "extension-1",
  kind: "extension",
  title: "Lyrics Provider",
  subtitle: "Fetches lyrics for the track that is playing.",
  icon: Icons.extension,
  run: noop,
  action: <Toggle checked label="Enable Lyrics Provider" onChange={noop} />,
};

const server: PaletteEntry = {
  key: "server-1",
  kind: "server",
  title: "Navidrome",
  subtitle: "music-player",
  icon: Icons.device,
  run: noop,
};

const station: PaletteEntry = {
  key: "radio-1",
  kind: "radio",
  title: "Radio Paradise",
  subtitle: "Eclectic · US · 320kbps",
  icon: Icons.broadcast,
  run: noop,
};

const entries = [track, album, artist, playlist, extension, server, station];

/**
 * The global search modal — the one place that searches everything: the
 * library, plus the playlists, extensions, servers and internet radio the
 * pages used to filter for themselves. Opened with `⌘K` or `/`.
 */
const meta: Meta<typeof CommandPalette> = {
  title: "Layout/CommandPalette",
  component: CommandPalette,
  args: {
    open: true,
    query: "daft",
    entries,
    selected: 0,
    onQueryChange: noop,
    onSelect: noop,
    onActivate: noop,
    onClose: noop,
  },
};

export default meta;

type Story = StoryObj<typeof CommandPalette>;

export const Default: Story = {};

/** Nothing typed yet: the hint row rather than an empty list. */
export const Empty: Story = {
  args: { query: "", entries: [] },
};

export const NoResults: Story = {
  args: { query: "zzzz", entries: [] },
};

/** A search is in flight; whatever has arrived stays on screen. */
export const Loading: Story = {
  args: { entries: [track, album], loading: true },
};

export const TracksOnly: Story = {
  args: {
    entries: [
      track,
      { ...track, key: "track-2", title: "Otherside", subtitle: "RHCP" },
      { ...track, key: "track-3", title: "Iowa", subtitle: "Slipknot" },
    ],
  },
};

/** Extension rows carry their own switch — the row opens, the switch toggles. */
export const ExtensionsWithActions: Story = {
  args: {
    query: "provider",
    entries: [
      extension,
      {
        ...extension,
        key: "extension-2",
        title: "Radio Browser",
        subtitle: "A browsable catalogue of internet radio stations.",
        action: <Toggle label="Enable Radio Browser" onChange={noop} />,
      },
    ],
  },
};

export const Servers: Story = {
  args: {
    query: "home",
    entries: [
      server,
      {
        ...server,
        key: "server-2",
        title: "Kitchen speaker",
        subtitle: "Cast · playing here",
      },
    ],
  },
};

/**
 * Stations come from radio-browser over the network, so they are the one kind
 * that waits for a second character before searching.
 */
export const RadioStations: Story = {
  args: {
    query: "paradise",
    entries: [
      station,
      {
        ...station,
        key: "radio-2",
        title: "SomaFM Groove Salad",
        subtitle: "Ambient · US · 128kbps",
      },
    ],
  },
};

/** The highlight is arrow-key driven; here it sits on the fourth row. */
export const SelectionMovedDown: Story = {
  args: { selected: 3 },
};
