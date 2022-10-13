import { linkTo } from "@storybook/addon-links";
import { ComponentMeta, ComponentStory } from "@storybook/react";
import _ from "lodash";
import ArtistDetails from "../Components/ArtistDetails";

export default {
  title: "Components/ArtistDetails",
  component: ArtistDetails,
} as ComponentMeta<typeof ArtistDetails>;

const Template: ComponentStory<typeof ArtistDetails> = (args: any) => (
  <ArtistDetails {...args} />
);

export const Default = Template.bind({});

Default.args = {
  artist: {
    name: "Daft Punk",
  },
  tracks: [
    ["Get Lucky", "Daft Punk", "Random Access Memories", "6:10"],
    ["Instant Crush", "Daft Punk", "Random Access Memories", "5:38"],
    ["Arround the World", "Daft Punk", "Homework", "7:10"],
  ],
  albums: [
    {
      id: "1",
      title: "Random Access Memories",
      artist: "Daft Punk",
      cover:
        "https://resources.tidal.com/images/09b59e6e/717e/43e3/b2e2/d2a153c24775/320x320.jpg",
    },
    {
      id: "2",
      title: "Tron: Legacy",
      artist: "Daft Punk",
      cover:
        "https://resources.tidal.com/images/866bb671/5ec8/4b45/8a60/50e51f5dbf10/320x320.jpg",
    },
    {
      id: "4",
      title: "Discovery",
      artist: "Daft Punk",
      cover:
        "https://resources.tidal.com/images/f853861c/0c5f/4e73/b608/eeb00618fe6f/320x320.jpg",
    },
  ],
  onBack: linkTo("Components/Artists", "Default"),
  onClickLibraryItem(item) {
    linkTo(
      `Components/${item
        .split("-")
        .map((x) => _.capitalize(x))
        .join("")}`,
      "Default"
    )();
  },
};
