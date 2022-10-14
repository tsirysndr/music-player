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
      {
        "#": "1",
        title: "Tic Tac",
        artist: "Future, Lil Uzi Vert",
        time: "3:09",
      },
      {
        "#": "2",
        title: "My Legacy",
        artist: "Future, Lil Uzi Vert",
        time: "3:13",
      },
      {
        "#": "3",
        title: "Heart In Pieces",
        artist: "Future, Lil Uzi Vert",
        time: "3:17",
      },
      {
        "#": "4",
        title: "Because of You",
        artist: "Future, Lil Uzi Vert",
        time: "3:37",
      },
      {
        "#": "5",
        title: "Bust a Move",
        artist: "Future, Lil Uzi Vert",
        time: "3:46",
      },
      {
        "#": "6",
        title: "Baby Sasuke",
        artist: "Future, Lil Uzi Vert",
        time: "3:30",
      },
      {
        "#": "7",
        title: "Stripes Like Burberry",
        artist: "Future, Lil Uzi Vert",
        time: "4:34",
      },
      {
        "#": "11",
        title: "Drankin N Smokin",
        artist: "Future, Lil Uzi Vert",
        time: "3:34",
      },
      {
        "#": "12",
        title: "Million Dollar Play",
        artist: "Future, Lil Uzi Vert",
        time: "2:47",
      },
      {
        "#": "13",
        title: "Plastic",
        artist: "Future, Lil Uzi Vert",
        time: "3:14",
      },
      {
        "#": "14",
        title: "That's It",
        artist: "Future, Lil Uzi Vert",
        time: "3:49",
      },
      {
        "#": "15",
        title: "Bought A Bad Bitch",
        artist: "Future, Lil Uzi Vert",
        time: "3:30",
      },
      {
        "#": "16",
        title: "Rockstar Chainz",
        artist: "Future, Lil Uzi Vert",
        time: "3:01",
      },
      {
        "#": "17",
        title: "Lullaby",
        artist: "Future, Lil Uzi Vert",
        time: "4:22",
      },
      {
        "#": "18",
        title: "She Never Been To Pluto",
        artist: "Future, Lil Uzi Vert",
        time: "3:24",
      },
      {
        "#": "19",
        title: "Off Dat",
        artist: "Future, Lil Uzi Vert",
        time: "3:05",
      },
      {
        "#": "20",
        title: "I Don't Wanna Break Up",
        artist: "Future, Lil Uzi Vert",
        time: "4:04",
      },
      {
        "#": "21",
        title: "Bankroll",
        artist: "Future, Lil Uzi Vert",
        time: "3:07",
      },
      {
        "#": "22",
        title: "Moment of Clarity",
        artist: "Future, Lil Uzi Vert",
        time: "3:42",
      },
      {
        "#": "23",
        title: "Patek",
        artist: "Future, Lil Uzi Vert",
        time: "4:41",
      },
      {
        "#": "24",
        title: "Over Your Head",
        artist: "Future, Lil Uzi Vert",
        time: "3:07",
      },
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
