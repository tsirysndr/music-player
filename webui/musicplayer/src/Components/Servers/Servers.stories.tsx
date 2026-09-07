import type { Meta, StoryObj } from "@storybook/react-vite";
import { noop } from "../../Stories/fixtures";
import Servers, { type ServerItem } from "./Servers";

const servers: ServerItem[] = [
  {
    id: "b6f0a1c2d3e4",
    name: "studio",
    kind: "music-player",
    address: "192.168.1.24:5040",
  },
  {
    id: "navidrome-1",
    name: "Navidrome",
    kind: "subsonic",
    address: "music.home.lan:4533",
  },
  {
    id: "jellyfin-1",
    name: "Jellyfin",
    kind: "jellyfin",
    address: "media.home.lan:8096",
  },
];

const castDevices: ServerItem[] = [
  { id: "cast-kitchen", name: "Kitchen speaker", kind: "chromecast", cast: true },
  { id: "cast-living", name: "Living room TV", kind: "chromecast", cast: true },
];

const meta: Meta<typeof Servers> = {
  title: "Pages/Servers",
  component: Servers,
  args: {
    servers,
    castDevices,
    onConnect: noop,
    onDisconnect: noop,
    onRefresh: noop,
  },
};

export default meta;

type Story = StoryObj<typeof Servers>;

export const Default: Story = {};

export const Loading: Story = {
  args: { servers: [], castDevices: [], loading: true },
};

export const Empty: Story = {
  args: { servers: [], castDevices: [] },
};

/** Nothing to cast to — the second section drops out rather than showing empty. */
export const ServersOnly: Story = {
  args: { castDevices: [] },
};

export const CastTargetsOnly: Story = {
  args: { servers: [] },
};

/** Handed off to a speaker: that row is selected and "Play here instead" appears. */
export const PlayingOnCastTarget: Story = {
  args: { connectedId: "cast-kitchen" },
};

export const PlayingOnAnotherServer: Story = {
  args: { connectedId: "navidrome-1" },
};

export const Connecting: Story = {
  args: { busyId: "jellyfin-1" },
};

export const Failed: Story = {
  args: {
    error: "Could not connect to Jellyfin: connection refused",
  },
};
