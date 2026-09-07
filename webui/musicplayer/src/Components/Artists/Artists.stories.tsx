import type { Meta, StoryObj } from "@storybook/react-vite";
import { artists, noop } from "../../Stories/fixtures";
import Artists from "./Artists";

const meta: Meta<typeof Artists> = {
  title: "Pages/Artists",
  component: Artists,
  args: {
    artists,
    filter: "",
    onFilter: noop,
    onPlayArtist: noop,
  },
};

export default meta;

type Story = StoryObj<typeof Artists>;

export const Default: Story = {};

export const Loading: Story = {
  args: { artists: [], loading: true },
};

export const Empty: Story = {
  args: { artists: [] },
};
