import { linkTo } from "@storybook/addon-links";
import { ComponentMeta, ComponentStory } from "@storybook/react";
import AlbumDetails from "./AlbumDetails";

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
        cover:
          "https://resources.tidal.com/images/fe6787d5/4ba5/4d3e/8576/48943ee6a768/320x320.jpg",
      },
      {
        "#": "2",
        title: "My Legacy",
        artist: "Future, Lil Uzi Vert",
        time: "3:13",
        cover:
          "https://resources.tidal.com/images/fe6787d5/4ba5/4d3e/8576/48943ee6a768/320x320.jpg",
      },
      {
        "#": "3",
        title: "Heart In Pieces",
        artist: "Future, Lil Uzi Vert",
        time: "3:17",
        cover:
          "https://resources.tidal.com/images/fe6787d5/4ba5/4d3e/8576/48943ee6a768/320x320.jpg",
      },
      {
        "#": "4",
        title: "Because of You",
        artist: "Future, Lil Uzi Vert",
        time: "3:37",
        cover:
          "https://resources.tidal.com/images/fe6787d5/4ba5/4d3e/8576/48943ee6a768/320x320.jpg",
      },
      {
        "#": "5",
        title: "Bust a Move",
        artist: "Future, Lil Uzi Vert",
        time: "3:46",
        cover:
          "https://resources.tidal.com/images/fe6787d5/4ba5/4d3e/8576/48943ee6a768/320x320.jpg",
      },
      {
        "#": "6",
        title: "Baby Sasuke",
        artist: "Future, Lil Uzi Vert",
        time: "3:30",
        cover:
          "https://resources.tidal.com/images/fe6787d5/4ba5/4d3e/8576/48943ee6a768/320x320.jpg",
      },
      {
        "#": "7",
        title: "Stripes Like Burberry",
        artist: "Future, Lil Uzi Vert",
        time: "4:34",
        cover:
          "https://resources.tidal.com/images/fe6787d5/4ba5/4d3e/8576/48943ee6a768/320x320.jpg",
      },
      {
        "#": "11",
        title: "Drankin N Smokin",
        artist: "Future, Lil Uzi Vert",
        time: "3:34",
        cover:
          "https://resources.tidal.com/images/fe6787d5/4ba5/4d3e/8576/48943ee6a768/320x320.jpg",
      },
      {
        "#": "12",
        title: "Million Dollar Play",
        artist: "Future, Lil Uzi Vert",
        time: "2:47",
        cover:
          "https://resources.tidal.com/images/fe6787d5/4ba5/4d3e/8576/48943ee6a768/320x320.jpg",
      },
      {
        "#": "13",
        title: "Plastic",
        artist: "Future, Lil Uzi Vert",
        time: "3:14",
        cover:
          "https://resources.tidal.com/images/fe6787d5/4ba5/4d3e/8576/48943ee6a768/320x320.jpg",
      },
      {
        "#": "14",
        title: "That's It",
        artist: "Future, Lil Uzi Vert",
        time: "3:49",
        cover:
          "https://resources.tidal.com/images/fe6787d5/4ba5/4d3e/8576/48943ee6a768/320x320.jpg",
      },
      {
        "#": "15",
        title: "Bought A Bad Bitch",
        artist: "Future, Lil Uzi Vert",
        time: "3:30",
        cover:
          "https://resources.tidal.com/images/fe6787d5/4ba5/4d3e/8576/48943ee6a768/320x320.jpg",
      },
      {
        "#": "16",
        title: "Rockstar Chainz",
        artist: "Future, Lil Uzi Vert",
        time: "3:01",
        cover:
          "https://resources.tidal.com/images/fe6787d5/4ba5/4d3e/8576/48943ee6a768/320x320.jpg",
      },
      {
        "#": "17",
        title: "Lullaby",
        artist: "Future, Lil Uzi Vert",
        time: "4:22",
        cover:
          "https://resources.tidal.com/images/fe6787d5/4ba5/4d3e/8576/48943ee6a768/320x320.jpg",
      },
      {
        "#": "18",
        title: "She Never Been To Pluto",
        artist: "Future, Lil Uzi Vert",
        time: "3:24",
        cover:
          "https://resources.tidal.com/images/fe6787d5/4ba5/4d3e/8576/48943ee6a768/320x320.jpg",
      },
      {
        "#": "19",
        title: "Off Dat",
        artist: "Future, Lil Uzi Vert",
        time: "3:05",
        cover:
          "https://resources.tidal.com/images/fe6787d5/4ba5/4d3e/8576/48943ee6a768/320x320.jpg",
      },
      {
        "#": "20",
        title: "I Don't Wanna Break Up",
        artist: "Future, Lil Uzi Vert",
        time: "4:04",
        cover:
          "https://resources.tidal.com/images/fe6787d5/4ba5/4d3e/8576/48943ee6a768/320x320.jpg",
      },
      {
        "#": "21",
        title: "Bankroll",
        artist: "Future, Lil Uzi Vert",
        time: "3:07",
        cover:
          "https://resources.tidal.com/images/fe6787d5/4ba5/4d3e/8576/48943ee6a768/320x320.jpg",
      },
      {
        "#": "22",
        title: "Moment of Clarity",
        artist: "Future, Lil Uzi Vert",
        time: "3:42",
        cover:
          "https://resources.tidal.com/images/fe6787d5/4ba5/4d3e/8576/48943ee6a768/320x320.jpg",
      },
      {
        "#": "23",
        title: "Patek",
        artist: "Future, Lil Uzi Vert",
        time: "4:41",
        cover:
          "https://resources.tidal.com/images/fe6787d5/4ba5/4d3e/8576/48943ee6a768/320x320.jpg",
      },
      {
        "#": "24",
        title: "Over Your Head",
        artist: "Future, Lil Uzi Vert",
        time: "3:07",
        cover:
          "https://resources.tidal.com/images/fe6787d5/4ba5/4d3e/8576/48943ee6a768/320x320.jpg",
      },
    ],
  },
  onBack: linkTo("Components/Albums", "Default"),
  nowPlaying: {},
};
