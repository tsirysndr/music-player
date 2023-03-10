import { ComponentMeta, ComponentStory } from "@storybook/react";
import Library from "./Library";

export default {
  title: "Components/Library",
  component: Library,
} as ComponentMeta<typeof Library>;

const Template: ComponentStory<typeof Library> = (args: any) => (
  <Library {...args} />
);

export const Default = Template.bind({});
