import { ComponentMeta, ComponentStory } from "@storybook/react";
import Albums from "../Components/Albums";

export default {
  title: "Components/Artists",
  component: Albums,
} as ComponentMeta<typeof Albums>;

const Template: ComponentStory<typeof Albums> = (args: any) => (
  <Albums {...args} />
);

export const Default = Template.bind({});
