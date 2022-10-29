import { ComponentMeta, ComponentStory } from "@storybook/react";
import ControlBar from "../Components/ControlBar";

export default {
  title: "Components/ControlBar",
  component: ControlBar,
  argTypes: {
    onPlay: { action: "play" },
    onPause: { action: "pause" },
    onNext: { action: "next" },
    onPrevious: { action: "previous" },
    onShuffle: { action: "shuffle" },
    onRepeat: { action: "repeat" },
  },
} as ComponentMeta<typeof ControlBar>;

const Template: ComponentStory<typeof ControlBar> = (args: any) => (
  <ControlBar {...args} />
);

export const Default = Template.bind({});

Default.args = {
  nowPlaying: {
    title: "Otherside",
    artist: "Red Hot Chilli Peppers",
    album: "Californication",
    duration: 255488.00659179688,
    progress: 123456.789,
    cover:
      "https://resources.tidal.com/images/543575fc/ad02/419b/ae61/671558dc019d/320x320.jpg",
    albumId: "671558dc019d",
  },
};

export const NotPlaying = Template.bind({});
