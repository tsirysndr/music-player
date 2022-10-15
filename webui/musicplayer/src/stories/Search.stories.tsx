import { ComponentMeta, ComponentStory } from "@storybook/react";
import Search from "../Components/Search";

export default {
  title: "Components/Search",
  component: Search,
} as ComponentMeta<typeof Search>;

const Template: ComponentStory<typeof Search> = (args: any) => (
  <Search {...args} />
);

export const Default = Template.bind({});
