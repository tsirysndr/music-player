import React from 'react';
import {ComponentMeta, ComponentStory} from '@storybook/react-native';
import Tracks from './Tracks';

export default {
  title: 'Tracks',
  component: Tracks,
  argTypes: {
    onGoBack: {action: 'onGoBack'},
  },
} as ComponentMeta<typeof Tracks>;

const Template: ComponentStory<typeof Tracks> = args => <Tracks {...args} />;

export const Default = Template.bind({});
