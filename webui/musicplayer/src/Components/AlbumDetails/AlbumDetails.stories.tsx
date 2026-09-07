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
    onQueueAlbum: noop,
    onLikeAlbum: noop,
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

/**
 * A two-disc album: the track table grows `DISC n` headers, exactly as the
 * desktop's does. A single-disc album gets none, so the common case is
 * unchanged.
 */
export const MultiDisc: Story = {
  args: {
    album: {
      ...albums[0],
      meta: "8 tracks",
      tracks: [
        ...tracks.map((track) => ({ ...track, discNumber: 1 })),
        ...tracks.map((track) => ({
          ...track,
          id: `${track.id}-d2`,
          discNumber: 2,
        })),
      ],
    },
  },
};

export const Loading: Story = {
  args: { album: undefined, loading: true },
};
