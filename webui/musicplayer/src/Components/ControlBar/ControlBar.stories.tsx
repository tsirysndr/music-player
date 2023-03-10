import { ComponentMeta, ComponentStory } from "@storybook/react";
import ControlBar from "./ControlBar";

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
    onRemoveTrackAt: { action: "removeTrackAt" },
    onPlayTrackAt: { action: "playTrackAt" },
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
  nextTracks: [
    {
      id: "5ce25a3dadabc01f94130d806294a296",
      title: "Can I",
      artist: "Kodak Black",
      album: "Lil Big Pac",
      cover:
        "https://resources.tidal.com/images/f36fa0ef/fe50/4c54/8b97/3c9f4f72f8e8/320x320.jpg",
      duration: 216.08399963378906,
      artistId: "c38e31502394622bbdfd3d82f6911eb3",
    },
    {
      id: "c2754b1edfcb8fb4dd34d94f4e42b622",
      title: "Too Many Years",
      artist: "Kodak Black, PnB Rock",
      album: "Lil Big Pac",
      cover:
        "https://resources.tidal.com/images/f36fa0ef/fe50/4c54/8b97/3c9f4f72f8e8/320x320.jpg",
      duration: 196.48699951171875,
      artistId: "a01003e6ac321328ad1e208bf41206f0",
    },
    {
      id: "89ab06746b363d3295a2bc297339c5be",
      title: "ALL THE TIME HIGH",
      artist: "Juicy J featuring Kaash Paige",
      album: "THE HUSTLE STILL CONTINUES (Deluxe)",
      cover:
        "https://resources.tidal.com/images/aeb814c6/7767/421f/a121/46fdc0895c53/320x320.jpg",
      duration: 120.14199829101562,
      artistId: "c96c630a2fd2edd0eec175c4f65e0329",
    },
  ],
  previousTracks: [
    {
      id: "93b7e61e27b4e5cab4c6ac4d45a4dc9f",
      title: "Sobriety Sucks",
      artist: "Yelawolf",
      album: "Trunk Muzic Returns (Deluxe Edition)",
      cover:
        "https://resources.tidal.com/images/3b8d6b45/b3e4/4f0c/a69d/2f31a1fceadf/320x320.jpg",
      duration: 255.00100708007812,
      artistId: "052dcb0741a91da6d26a35600ec46cd9",
    },
  ],
};

export const NotPlaying = Template.bind({});
