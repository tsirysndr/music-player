import React from 'react';
import {ComponentMeta, ComponentStory} from '@storybook/react-native';
import MiniPlayer from './MiniPlayer';

export default {
  title: 'MiniPlayer',
  component: MiniPlayer,
} as ComponentMeta<typeof MiniPlayer>;

const Template: ComponentStory<typeof MiniPlayer> = args => (
  <MiniPlayer {...args} />
);

export const Default = Template.bind({});
