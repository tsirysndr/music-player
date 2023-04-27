import React from 'react';
import {ComponentMeta, ComponentStory} from '@storybook/react-native';
import TrackRow from './TrackRow';
import {tracks} from '../../Mocks/Tracks';

export default {
  title: 'TrackRow',
  component: TrackRow,
  argTypes: {
    onPlay: {action: 'onPlay'},
  },
} as ComponentMeta<typeof TrackRow>;

const Template: ComponentStory<typeof TrackRow> = args => (
  <TrackRow {...args} />
);

export const Default = Template.bind({});
Default.args = {
  track: tracks[0],
};
