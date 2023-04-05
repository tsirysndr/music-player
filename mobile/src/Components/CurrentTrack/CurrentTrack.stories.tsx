import React from 'react';
import {ComponentMeta, ComponentStory} from '@storybook/react-native';
import CurrentTrack from './CurrentTrack';
import {tracks} from '../../Mocks/Tracks';

export default {
  title: 'CurrentTrack',
  component: CurrentTrack,
  argTypes: {
    onPageSelected: {action: 'onPageSelected'},
  },
} as ComponentMeta<typeof CurrentTrack>;

const Template: ComponentStory<typeof CurrentTrack> = args => (
  <CurrentTrack {...args} />
);

export const Default = Template.bind({});

Default.args = {
  track: tracks[0],
  initialPage: 0,
  tracks,
};

export const NoCover = Template.bind({});

NoCover.args = {
  track: tracks[3],
};
