import { ComponentMeta, ComponentStory } from "@storybook/react";
import Tracks from "./Tracks";

export default {
  title: "Components/Tracks",
  component: Tracks,
  argTypes: {
    onPlay: { action: "play" },
    onPause: { action: "pause" },
    onNext: { action: "next" },
    onPrevious: { action: "previous" },
    onShuffle: { action: "shuffle" },
    onRepeat: { action: "repeat" },
  },
} as ComponentMeta<typeof Tracks>;

const Template: ComponentStory<typeof Tracks> = (args: any) => (
  <Tracks {...args} />
);

export const Default = Template.bind({});

Default.args = {
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
};
