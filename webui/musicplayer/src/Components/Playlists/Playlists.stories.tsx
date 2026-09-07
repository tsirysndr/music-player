import type { Meta, StoryObj } from "@storybook/react-vite";
import { noop, playlists } from "../../Stories/fixtures";
import Playlists from "./Playlists";

const meta: Meta<typeof Playlists> = {
  title: "Pages/Playlists",
  component: Playlists,
  args: {
    playlists,
    folders: [
      { id: "f1", name: "Moods" },
      { id: "f2", name: "Work" },
    ],
    onCreatePlaylist: noop,
    onCreateFolder: noop,
    onEditPlaylist: noop,
    onDeletePlaylist: noop,
    onPlayPlaylist: noop,
  },
};

export default meta;

type Story = StoryObj<typeof Playlists>;

export const Default: Story = {};

export const NoFolders: Story = {
  args: { folders: [] },
};

export const Empty: Story = {
  args: { playlists: [], folders: [] },
};
