import { ComponentMeta, ComponentStory } from "@storybook/react";
import Playlists from "../Components/Playlists";

export default {
  title: "Components/Playlists",
  component: Playlists,
} as ComponentMeta<typeof Playlists>;

const Template: ComponentStory<typeof Playlists> = (args: any) => (
  <Playlists {...args} />
);

export const Default = Template.bind({});
