import { linkTo } from "@storybook/addon-links";
import { ComponentMeta, ComponentStory } from "@storybook/react";
import _ from "lodash";
import PlayQueue from "../Components/PlayQueue";

export default {
  title: "Components/PlayQueue",
  component: PlayQueue,
} as ComponentMeta<typeof PlayQueue>;

const Template: ComponentStory<typeof PlayQueue> = (args: any) => (
  <PlayQueue {...args} />
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
