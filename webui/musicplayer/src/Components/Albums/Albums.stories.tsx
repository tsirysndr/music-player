import type { Meta, StoryObj } from "@storybook/react-vite";
import { albums, noop } from "../../Stories/fixtures";
import Albums from "./Albums";

const meta: Meta<typeof Albums> = {
  title: "Pages/Albums",
  component: Albums,
  args: {
    albums,
    filter: "",
    onFilter: noop,
    onPlayAlbum: noop,
    onShuffleAlbum: noop,
  },
};

export default meta;

type Story = StoryObj<typeof Albums>;

export const Default: Story = {};

export const Loading: Story = {
  args: { albums: [], loading: true },
};

export const Empty: Story = {
  args: { albums: [] },
};
