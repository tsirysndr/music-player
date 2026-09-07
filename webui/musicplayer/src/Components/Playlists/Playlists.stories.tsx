import type { Meta, StoryObj } from "@storybook/react-vite";
import Playlists from "./Playlists";

const meta: Meta<typeof Playlists> = {
  title: "Components/Playlists",
  component: Playlists,
};

export default meta;

export const Default: StoryObj<typeof Playlists> = {};
