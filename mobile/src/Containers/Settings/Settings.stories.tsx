import React from 'react';
import {ComponentMeta, ComponentStory} from '@storybook/react-native';
import Settings from './Settings';

export default {
  title: 'Settings',
  component: Settings,
} as ComponentMeta<typeof Settings>;

const Template: ComponentStory<typeof Settings> = args => (
  <Settings {...args} />
);

export const Default = Template.bind({});
