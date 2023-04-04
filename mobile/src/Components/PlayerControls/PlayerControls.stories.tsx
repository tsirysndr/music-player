import React from 'react';
import {ComponentMeta, ComponentStory} from '@storybook/react-native';
import PlayerControls from './PlayerControls';

export default {
  title: 'PlayerControls',
  component: PlayerControls,
} as ComponentMeta<typeof PlayerControls>;

const Template: ComponentStory<typeof PlayerControls> = args => (
  <PlayerControls {...args} />
);

export const Default = Template.bind({});
