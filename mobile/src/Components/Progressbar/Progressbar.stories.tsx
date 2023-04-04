import React from 'react';
import {ComponentMeta, ComponentStory} from '@storybook/react-native';
import Progressbar from './Progressbar';

export default {
  title: 'Progressbar',
  component: Progressbar,
  argTypes: {
    progress: {control: 'number'},
    onValueChange: {action: 'onValueChange'},
  },
} as ComponentMeta<typeof Progressbar>;

const Template: ComponentStory<typeof Progressbar> = args => (
  <Progressbar {...args} />
);

export const Default = Template.bind({});

Default.args = {
  progress: 40,
  thumbTintColor: '#ab28fc',
};
