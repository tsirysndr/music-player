import { ComponentMeta, ComponentStory } from "@storybook/react";
import ContextMenu from "../Components/ContextMenu";

export default {
  title: "Components/ContextMenu",
  component: ContextMenu,
} as ComponentMeta<typeof ContextMenu>;

const Template: ComponentStory<typeof ContextMenu> = (args: any) => (
  <ContextMenu {...args} />
);

export const Default = Template.bind({});

Default.args = {
  liked: false,
};
