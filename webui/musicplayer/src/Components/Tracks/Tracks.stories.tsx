import type { Meta, StoryObj } from "@storybook/react-vite";
import Tracks from "./Tracks";

const meta: Meta<typeof Tracks> = {
  title: "Components/Tracks",
  component: Tracks,
};

export default meta;

export const Default: StoryObj<typeof Tracks> = {
  args: {
  tracks: [
    {
      title: "Otherside",
      artist: "Red Hot Chilli Peppers",
      album: "Californication",
      time: "4:15",
    },
    {
      title: "Road Trippin'",
      artist: "Red Hot Chilli Peppers",
      album: "Californication",
      time: "3:25",
    },
  ],
  nowPlaying: {},
  currentCastDevice: undefined,
},
};
