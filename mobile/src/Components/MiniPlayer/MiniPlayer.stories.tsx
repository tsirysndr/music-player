import React from 'react';
import {ComponentMeta, ComponentStory} from '@storybook/react-native';
import MiniPlayer from './MiniPlayer';
import {tracks} from '../../Mocks/Tracks';

export default {
  title: 'MiniPlayer',
  component: MiniPlayer,
  argTypes: {
    playing: {control: 'boolean'},
    onPlay: {action: 'onPlay'},
    onPause: {action: 'onPause'},
    onSkipNext: {action: 'onSkipNext'},
    progress: {control: 'number'},
  },
} as ComponentMeta<typeof MiniPlayer>;

const Template: ComponentStory<typeof MiniPlayer> = args => (
  <MiniPlayer {...args} />
);

export const Default = Template.bind({});

Default.args = {
  progress: 40,
  track: tracks[0],
};
