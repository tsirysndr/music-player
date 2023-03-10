import { linkTo } from "@storybook/addon-links";
import { ComponentMeta, ComponentStory } from "@storybook/react";
import Artists from "./Artists";

export default {
  title: "Components/Artists",
  component: Artists,
  argTypes: {
    onPlay: { action: "play" },
    onPause: { action: "pause" },
    onNext: { action: "next" },
    onPrevious: { action: "previous" },
    onShuffle: { action: "shuffle" },
    onRepeat: { action: "repeat" },
  },
} as ComponentMeta<typeof Artists>;

const Template: ComponentStory<typeof Artists> = (args: any) => (
  <Artists {...args} />
);

export const Default = Template.bind({});

Default.args = {
  onClickArtist(artist) {
    linkTo("Components/ArtistDetails", "Default")();
  },
  artists: [
    {
      id: "1",
      name: "Red Hot Chilli Peppers",
      cover:
        "https://resources.tidal.com/images/9be9e74c/1b06/479a/a79e/5c62733d74ac/320x320.jpg",
    },
    {
      id: "2",
      name: "Daft Punk",
      cover:
        "https://resources.tidal.com/images/f87d9afc/075e/43f4/bbbc/7770b46cb8aa/320x320.jpg",
    },
    {
      id: "3",
      name: "Slipknot",
      cover:
        "https://resources.tidal.com/images/c3672adf/ca16/4db3/a375/64904d1f7d39/320x320.jpg",
    },
    {
      id: "4",
      name: "Big K.R.I.T.",
      cover:
        "https://resources.tidal.com/images/2021e89d/f4bd/4d41/9af7/da47dc7564e8/320x320.jpg",
    },
    {
      id: "4",
      name: "Oasis",
    },
  ],
  currentCastDevice: undefined,
};
