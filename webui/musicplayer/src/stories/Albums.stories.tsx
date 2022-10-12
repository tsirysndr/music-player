import { ComponentMeta, ComponentStory } from "@storybook/react";
import { linkTo } from "@storybook/addon-links";
import Albums from "../Components/Albums/Albums";
import _ from "lodash";

export default {
  title: "Components/Albums",
  component: Albums,
} as ComponentMeta<typeof Albums>;

const albums = [
  {
    id: "1",
    title: "The End, So Far",
    artist: "Slipknot",
    cover:
      "https://resources.tidal.com/images/b80f3852/73e5/43d6/90ea/021d1fad2bbe/320x320.jpg",
  },
  {
    id: "2",
    title: "One Cold Night (Live)",
    artist: "Seether",
    cover:
      "https://resources.tidal.com/images/f6f5f0a6/dc95/4561/9ca6/6ba1e0f6a062/320x320.jpg",
  },
  {
    id: "3",
    title: "4Eva N A Day",
    artist: "Big K.R.I.T.",
    cover:
      "https://resources.tidal.com/images/3d58ba7f/8e6d/4a2e/a2e4/793495b8916a/320x320.jpg",
  },
  {
    id: "4",
    title: "Pluto x Baby Pluto (Deluxe)",
    artist: "Future, Lil Uzi Vert",
    cover:
      "https://resources.tidal.com/images/fe6787d5/4ba5/4d3e/8576/48943ee6a768/320x320.jpg",
  },
  {
    id: "5",
    title: "The Acoustic Sessions, Vol. 2",
    artist: "Smile Empty Soul",
    cover:
      "https://resources.tidal.com/images/06a15286/8b80/40a5/bd40/187e99e2e80d/320x320.jpg",
  },
  {
    id: "6",
    title: "Black Summer",
    artist: "Red Hot Chilli Peppers",
    cover:
      "https://resources.tidal.com/images/0bb1f23f/94a3/4211/8b6a/be57904ea709/320x320.jpg",
  },
  {
    id: "7",
    title: "Californication (Deluxe Edition)",
    artist: "Red Hot Chilli Peppers",
    cover:
      "https://resources.tidal.com/images/543575fc/ad02/419b/ae61/671558dc019d/320x320.jpg",
  },
  {
    id: "8",
    title: "Ghetto Lenny's Love Songs",
    artist: "SAINt JHN",
    cover:
      "https://resources.tidal.com/images/f2d921e6/6723/4a7c/a3c0/f630c9ce94eb/320x320.jpg",
  },
];

const Template: ComponentStory<typeof Albums> = (args: any) => (
  <Albums {...args} />
);

export const Default = Template.bind({});

Default.args = {
  albums,
  onClickAlbum(album) {
    linkTo("Components/AlbumDetails", "Default")();
  },
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
