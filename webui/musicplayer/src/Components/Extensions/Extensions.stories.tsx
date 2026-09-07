import type { Meta, StoryObj } from "@storybook/react-vite";
import { noop } from "../../Stories/fixtures";
import Extensions, { type ExtensionItem } from "./Extensions";

const extensions: ExtensionItem[] = [
  {
    id: "fm.atradio.lyrics-provider",
    name: "Lyrics Provider",
    version: "1.2.0",
    author: "atradio",
    description: "Fetches lyrics for the track that is playing.",
    homepage: "https://example.com",
    repository: "https://github.com/example/lyrics",
    license: "MIT",
    logo: "",
    topics: ["lyrics", "metadata"],
    capabilities: ["metadata"],
    allowedHosts: ["api.lyrics.example.com"],
    libraryRead: false,
    status: "enabled",
    path: "/home/you/.config/music-player/extensions/lyrics-provider",
  },
  {
    id: "fm.atradio.radio-browser",
    name: "Radio Browser",
    version: "0.4.1",
    author: "atradio",
    description: "A browsable catalogue of internet radio stations.",
    homepage: "",
    repository: "https://github.com/example/radio-browser",
    license: "Apache-2.0",
    logo: "",
    topics: ["radio", "source"],
    capabilities: ["source"],
    allowedHosts: ["all.api.radio-browser.info"],
    libraryRead: true,
    status: "enabled",
    path: "/home/you/.config/music-player/extensions/radio-browser",
  },
  {
    id: "com.example.mood",
    name: "Mood Predicate",
    version: "0.1.0",
    author: "Someone",
    description:
      "Adds `ext:mood` and `ext:era` to smart-playlist filters, derived from genre and year.",
    homepage: "",
    repository: "",
    license: "MIT",
    logo: "",
    topics: ["smart-playlists"],
    capabilities: ["predicates", "commands"],
    allowedHosts: [],
    libraryRead: false,
    status: "disabled",
    path: "/home/you/.config/music-player/extensions/mood-predicate",
  },
];

const meta: Meta<typeof Extensions> = {
  title: "Pages/Extensions",
  component: Extensions,
  args: {
    extensions,
    status: "all",
    onStatusFilter: noop,
    onToggle: noop,
    onRescan: noop,
  },
};

export default meta;

type Story = StoryObj<typeof Extensions>;

export const Default: Story = {};

export const Loading: Story = {
  args: { extensions: [], loading: true },
};

export const Empty: Story = {
  args: { extensions: [] },
};

export const NoMatchingStatus: Story = {
  args: { extensions: [], status: "disabled" },
};

export const Failed: Story = {
  args: {
    extensions: [],
    error: "Failed to fetch",
  },
};
