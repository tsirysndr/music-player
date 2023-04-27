import React from 'react';
import {ComponentMeta, ComponentStory} from '@storybook/react-native';
import PlayerControls from './PlayerControls';

export default {
  title: 'PlayerControls',
  component: PlayerControls,
  argTypes: {
    onPlay: {action: 'onPlay'},
    onPause: {action: 'onPause'},
    onSkipNext: {action: 'onSkipNext'},
    onSkipPrevious: {action: 'onSkipPrevious'},
    onSeek: {action: 'onSeek'},
    onShuffle: {action: 'onShuffle'},
    onRepeat: {action: 'onRepeat'},
  },
} as ComponentMeta<typeof PlayerControls>;

const Template: ComponentStory<typeof PlayerControls> = args => (
  <PlayerControls {...args} />
);

export const Default = Template.bind({});

Default.args = {
  playerState: {
    isPlaying: false,
    isShuffling: false,
    isRepeating: false,
    progress: 30,
    time: '01:13',
    duration: '03:45',
  },
};
