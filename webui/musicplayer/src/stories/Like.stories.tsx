import { ComponentMeta, ComponentStory } from "@storybook/react";
import Like from "../Components/Like";

export default {
  title: "Components/Like",
  component: Like,
} as ComponentMeta<typeof Like>;

const Template: ComponentStory<typeof Like> = (args: any) => <Like {...args} />;

export const Default = Template.bind({});
