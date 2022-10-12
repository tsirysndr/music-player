import { linkTo } from "@storybook/addon-links";
import { ComponentMeta, ComponentStory } from "@storybook/react";
import _ from "lodash";
import Tracks from "../Components/Tracks";

export default {
  title: "Components/Tracks",
  component: Tracks,
} as ComponentMeta<typeof Tracks>;

const Template: ComponentStory<typeof Tracks> = (args: any) => (
  <Tracks {...args} />
);

export const Default = Template.bind({});

Default.args = {
  tracks: [
    ["Otherside", "Red Hot Chilli Peppers", "Californication", "4:15"],
    ["Road Trippin'", "Red Hot Chilli Peppers", "Californication", "3:25"],
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
