import React from 'react';
import {ComponentMeta, ComponentStory} from '@storybook/react-native';
import Songs from './Songs';
import {tracks} from '../../Mocks/Tracks';

export default {
  title: 'Songs',
  component: Songs,
  argTypes: {
    onPressTrack: {action: 'onPressTrack'},
    onSeeAll: {action: 'onSeeAll'},
  },
} as ComponentMeta<typeof Songs>;

const Template: ComponentStory<typeof Songs> = args => <Songs {...args} />;

export const Default = Template.bind({});

Default.args = {
  tracks,
};
