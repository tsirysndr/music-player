import type { Meta, StoryObj } from "@storybook/react-vite";
import { noop } from "../../Stories/fixtures";
import Servers, { type ServerItem, type SourceKind } from "./Servers";

/** As the daemon's provider registry describes itself. */
const kinds: SourceKind[] = [
  {
    kind: "subsonic",
    displayName: "Subsonic / Navidrome",
    needsCredentials: true,
    defaultPort: 4533,
  },
  {
    kind: "jellyfin",
    displayName: "Jellyfin",
    needsCredentials: true,
    defaultPort: 8096,
  },
  {
    kind: "music-player",
    displayName: "music-player",
    needsCredentials: false,
    defaultPort: 5053,
  },
  { kind: "kodi", displayName: "Kodi", needsCredentials: true, defaultPort: 8080 },
  {
    kind: "plex",
    displayName: "Plex (token in the password field)",
    needsCredentials: true,
    defaultPort: 32400,
  },
  {
    kind: "rocksky",
    displayName: "Rocksky",
    needsCredentials: true,
    defaultPort: 443,
    fixedUrl: "https://navidrome.rocksky.app",
  },
];

const servers: ServerItem[] = [
  {
    id: "a1",
    kind: "subsonic",
    name: "Living room NAS",
    url: "http://192.168.1.10:4533",
    username: "tsiry",
    connected: false,
  },
  {
    id: "b2",
    kind: "jellyfin",
    name: "Media",
    url: "http://media.home.lan:8096",
    username: "tsiry",
    connected: false,
  },
  {
    id: "c3",
    kind: "music-player",
    name: "studio",
    url: "http://192.168.1.24:5053",
    connected: false,
  },
];

const meta: Meta<typeof Servers> = {
  title: "Pages/Servers",
  component: Servers,
  args: {
    servers,
    kinds,
    onAdd: noop,
    onConnect: noop,
    onDisconnect: noop,
    onDelete: noop,
  },
};

export default meta;

type Story = StoryObj<typeof Servers>;

export const Default: Story = {};

export const Loading: Story = {
  args: { servers: [], loading: true },
};

/** Nothing saved yet — adding one is the only thing to do here. */
export const Empty: Story = {
  args: { servers: [] },
};

/**
 * Connected: the row is selected and says so, and every library screen is now
 * reading from that server. Whatever is playing kept playing.
 */
export const Connected: Story = {
  args: {
    servers: servers.map((server) =>
      server.id === "a1" ? { ...server, connected: true } : server
    ),
  },
};

export const Connecting: Story = {
  args: { busyId: "b2" },
};

export const Failed: Story = {
  args: {
    error: "authentication failed: wrong username or password",
  },
};

/** An unreachable server the user can still delete or retry. */
export const UnreachableServer: Story = {
  args: {
    servers: [servers[0]],
    error: "error sending request for url (http://192.168.1.10:4533/rest/ping)",
  },
};
