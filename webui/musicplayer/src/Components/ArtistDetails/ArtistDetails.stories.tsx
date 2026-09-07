import type { Meta, StoryObj } from "@storybook/react-vite";
import { albums, artists, noop, recentPlaylists, tracks } from "../../Stories/fixtures";
import ArtistDetails from "./ArtistDetails";

const meta: Meta<typeof ArtistDetails> = {
  title: "Pages/ArtistDetails",
  component: ArtistDetails,
  args: {
    artist: artists[0],
    albums,
    tracks,
    recentPlaylists,
    onBack: noop,
    onPlayArtist: noop,
    onPlayAlbum: noop,
    onPlayNext: noop,
    onToggleLike: noop,
    onAddTrackToPlaylist: noop,
  },
};

export default meta;

type Story = StoryObj<typeof ArtistDetails>;

export const Default: Story = {};

/** An artist with songs but no albums of their own in the library. */
export const SongsOnly: Story = {
  args: { albums: [] },
};

export const Loading: Story = {
  args: { artist: undefined, loading: true },
};
