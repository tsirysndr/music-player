import { ComponentMeta, ComponentStory } from "@storybook/react";
import Filter from "../Components/Filter";

export default {
  title: "Components/Filter",
  component: Filter,
} as ComponentMeta<typeof Filter>;

const Template: ComponentStory<typeof Filter> = (args: any) => (
  <Filter {...args} />
);

export const Default = Template.bind({});
