import { ComponentMeta, ComponentStory } from "@storybook/react";
import ControlBar from "../Components/ControlBar";

export default {
  title: "Components/ControlBar",
  component: ControlBar,
} as ComponentMeta<typeof ControlBar>;

const Template: ComponentStory<typeof ControlBar> = (args: any) => (
  <ControlBar {...args} />
);

export const Default = Template.bind({});
