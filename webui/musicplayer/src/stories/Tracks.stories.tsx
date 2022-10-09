import { ComponentMeta, ComponentStory } from "@storybook/react";
import Tracks from "../Components/Tracks";

export default {
  title: "Components/Tracks",
  component: Tracks,
} as ComponentMeta<typeof Tracks>;

const Template: ComponentStory<typeof Tracks> = (args: any) => (
  <Tracks {...args} />
);

export const Default = Template.bind({});
