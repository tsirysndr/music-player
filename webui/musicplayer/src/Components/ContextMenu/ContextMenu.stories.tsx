import type { Meta, StoryObj } from "@storybook/react-vite";
import ContextMenu from "./ContextMenu";

const meta: Meta<typeof ContextMenu> = {
  title: "Components/ContextMenu",
  component: ContextMenu,
};

export default meta;

export const Default: StoryObj<typeof ContextMenu> = {
  args: {
  liked: false,
  track: {
    title: "Drankin N Smokin",
    artist: "Future, Lil Uzi Vert",
    time: "3:34",
    cover:
      "https://resources.tidal.com/images/fe6787d5/4ba5/4d3e/8576/48943ee6a768/320x320.jpg",
  },
},
};

export const NoAlbumCover: StoryObj<typeof ContextMenu> = {
  args: {
  liked: false,
  track: {
    title: "Million Dollar Play",
    artist: "Future, Lil Uzi Vert",
    time: "2:47",
  },
},
};


