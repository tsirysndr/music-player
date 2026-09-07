import type { Meta, StoryObj } from "@storybook/react-vite";
import { noop, recentPlaylists, tracks } from "../../Stories/fixtures";
import Tracks from "./Tracks";

const meta: Meta<typeof Tracks> = {
  title: "Pages/Tracks",
  component: Tracks,
  args: {
    tracks,
    filter: "",
    recentPlaylists,
    onFilter: noop,
    onPlayTrack: noop,
    onPlayNext: noop,
    onToggleLike: noop,
    onAddTrackToPlaylist: noop,
  },
};

export default meta;

type Story = StoryObj<typeof Tracks>;

export const Default: Story = {};

/** The current row is tinted and its number becomes a note glyph. */
export const Playing: Story = {
  args: { currentTrackId: tracks[1].id },
};

export const Loading: Story = {
  args: { tracks: [], loading: true },
};

export const Empty: Story = {
  args: { tracks: [] },
};

/** What a filter that matches nothing looks like. */
export const NoMatches: Story = {
  args: { tracks: [], filter: "zzzz" },
};
