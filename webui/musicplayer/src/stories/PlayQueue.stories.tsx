import { ComponentMeta, ComponentStory } from "@storybook/react";
import PlayQueue from "../Components/PlayQueue";

export default {
  title: "Components/PlayQueue",
  component: PlayQueue,
} as ComponentMeta<typeof PlayQueue>;

const Template: ComponentStory<typeof PlayQueue> = (args: any) => (
  <PlayQueue {...args} />
);

export const Default = Template.bind({});
