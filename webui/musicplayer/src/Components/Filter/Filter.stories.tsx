import { ComponentMeta, ComponentStory } from "@storybook/react";
import Filter from "./Filter";

export default {
  title: "Components/Filter",
  component: Filter,
  argTypes: {
    onChange: { action: "onChange" },
  },
} as ComponentMeta<typeof Filter>;

const Template: ComponentStory<typeof Filter> = (args: any) => (
  <Filter {...args} />
);

export const Default = Template.bind({});
