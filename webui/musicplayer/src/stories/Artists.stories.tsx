import { ComponentMeta, ComponentStory } from "@storybook/react";
import Artists from "../Components/Artists";

export default {
  title: "Components/Artists",
  component: Artists,
} as ComponentMeta<typeof Artists>;

const Template: ComponentStory<typeof Artists> = (args: any) => (
  <Artists {...args} />
);

export const Default = Template.bind({});
