import React from 'react';
import {ComponentMeta, ComponentStory} from '@storybook/react-native';
import PlayQueue from './PlayQueue';
import {tracks} from '../../Mocks/Tracks';

export default {
  title: 'PlayQueue',
  component: PlayQueue,
  argTypes: {
    currentTrack: {control: 'object'},
    nextTracks: {control: 'array'},
    onPlayItem: {action: 'onPlayItem'},
  },
} as ComponentMeta<typeof PlayQueue>;

const Template: ComponentStory<typeof PlayQueue> = args => (
  <PlayQueue {...args} />
);

export const Default = Template.bind({});

Default.args = {
  currentTrack: tracks[0],
  nextTracks: tracks.slice(1),
};
