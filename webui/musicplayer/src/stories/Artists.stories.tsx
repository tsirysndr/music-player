import { linkTo } from "@storybook/addon-links";
import { ComponentMeta, ComponentStory } from "@storybook/react";
import _ from "lodash";
import Artists from "../Components/Artists";

export default {
  title: "Components/Artists",
  component: Artists,
} as ComponentMeta<typeof Artists>;

const Template: ComponentStory<typeof Artists> = (args: any) => (
  <Artists {...args} />
);

export const Default = Template.bind({});

Default.args = {
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
