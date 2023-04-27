import React from 'react';
import {ComponentMeta, ComponentStory} from '@storybook/react-native';
import Player from './Player';
import {tracks} from '../../Mocks/Tracks';

export default {
  title: 'Player',
  component: Player,
} as ComponentMeta<typeof Player>;

const Template: ComponentStory<typeof Player> = args => <Player {...args} />;

export const Default = Template.bind({});

Default.args = {
  track: tracks[0],
};

export const NoCover = Template.bind({});

NoCover.args = {
  track: tracks[3],
};
