import { ComponentMeta, ComponentStory } from "@storybook/react";
import ContextMenu from "../Components/ContextMenu";

export default {
  title: "Components/ContextMenu",
  component: ContextMenu,
} as ComponentMeta<typeof ContextMenu>;

const Template: ComponentStory<typeof ContextMenu> = (args: any) => (
  <ContextMenu {...args} />
);

export const Default = Template.bind({});

export const NoAlbumCover = Template.bind({});

Default.args = {
  liked: false,
  track: {
    title: "Drankin N Smokin",
    artist: "Future, Lil Uzi Vert",
    time: "3:34",
    cover:
      "https://resources.tidal.com/images/fe6787d5/4ba5/4d3e/8576/48943ee6a768/320x320.jpg",
  },
};

NoAlbumCover.args = {
  liked: false,
  track: {
    title: "Million Dollar Play",
    artist: "Future, Lil Uzi Vert",
    time: "2:47",
  },
};
