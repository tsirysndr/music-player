import { linkTo } from "@storybook/addon-links";
import { ComponentMeta, ComponentStory } from "@storybook/react";
import _ from "lodash";
import AlbumDetails from "../Components/AlbumDetails";

export default {
  title: "Components/AlbumDetails",
  component: AlbumDetails,
} as ComponentMeta<typeof AlbumDetails>;

const Template: ComponentStory<typeof AlbumDetails> = (args: any) => (
  <AlbumDetails {...args} />
);

export const Default = Template.bind({});

Default.args = {
  album: {
    id: "4",
    title: "Pluto x Baby Pluto (Deluxe)",
    artist: "Future, Lil Uzi Vert",
    cover:
      "https://resources.tidal.com/images/fe6787d5/4ba5/4d3e/8576/48943ee6a768/320x320.jpg",
    tracks: [
      ["1", "Tic Tac", "Future, Lil Uzi Vert", "3:09"],
      ["2", "My Legacy", "Future, Lil Uzi Vert", "3:13"],
      ["3", "Heart In Pieces", "Future, Lil Uzi Vert", "3:17"],
      ["4", "Because of You", "Future, Lil Uzi Vert", "3:37"],
      ["5", "Bust a Move", "Future, Lil Uzi Vert", "3:46"],
      ["6", "Baby Sasuke", "Future, Lil Uzi Vert", "3:30"],
      ["7", "Stripes Like Burberry", "Future, Lil Uzi Vert", "4:34"],
      ["8", "Marni On Me", "Future, Lil Uzi Vert", "2:13"],
      ["9", "Sleeping On The Floor", "Future, Lil Uzi Vert", "3:47"],
      ["10", "Real Baby Pluto", "Future, Lil Uzi Vert", "3:17"],
      ["11", "Drankin N Smokin", "Future, Lil Uzi Vert", "3:34"],
      ["12", "Million Dollar Play", "Future, Lil Uzi Vert", "2:47"],
      ["13", "Plastic", "Future, Lil Uzi Vert", "3:14"],
      ["14", "That's It", "Future, Lil Uzi Vert", "3:49"],
      ["15", "Bought A Bad Bitch", "Future, Lil Uzi Vert", "3:30"],
      ["16", "Rockstar Chainz", "Future, Lil Uzi Vert", "3:01"],
      ["17", "Lullaby", "Future, Lil Uzi Vert", "4:22"],
      ["18", "She Never Been To Pluto", "Future, Lil Uzi Vert", "3:24"],
      ["19", "Off Dat", "Future, Lil Uzi Vert", "3:05"],
      ["20", "I Don't Wanna Break Up", "Future, Lil Uzi Vert", "4:04"],
      ["21", "Bankroll", "Future, Lil Uzi Vert", "3:07"],
      ["22", "Moment of Clarity", "Future, Lil Uzi Vert", "3:42"],
      ["23", "Patek", "Future, Lil Uzi Vert", "4:41"],
      ["24", "Over Your Head", "Future, Lil Uzi Vert", "3:07"],
    ],
  },
  onBack: linkTo("Components/Albums", "Default"),
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
