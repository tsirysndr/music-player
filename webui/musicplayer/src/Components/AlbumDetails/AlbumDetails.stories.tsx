import type { Meta, StoryObj } from "@storybook/react-vite";
import { albums, noop, recentPlaylists, tracks } from "../../Stories/fixtures";
import AlbumDetails from "./AlbumDetails";

const meta: Meta<typeof AlbumDetails> = {
  title: "Pages/AlbumDetails",
  component: AlbumDetails,
  args: {
    album: {
      ...albums[0],
      meta: `${tracks.length} tracks`,
      tracks,
    },
    recentPlaylists,
    onBack: noop,
    onPlayAlbum: noop,
    onPlayNext: noop,
    onToggleLike: noop,
    onAddTrackToPlaylist: noop,
  },
};

export default meta;

type Story = StoryObj<typeof AlbumDetails>;

export const Default: Story = {};

export const Playing: Story = {
  args: { currentTrackId: tracks[0].id },
};

/** No cover art: the skin's placeholder and disc glyph stand in. */
export const WithoutArtwork: Story = {
  args: {
    album: { ...albums[3], meta: `${tracks.length} tracks`, tracks },
  },
};

export const Loading: Story = {
  args: { album: undefined, loading: true },
};
