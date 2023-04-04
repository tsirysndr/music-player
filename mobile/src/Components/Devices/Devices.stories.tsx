import React from 'react';
import {ComponentMeta, ComponentStory} from '@storybook/react-native';
import Devices from './Devices';

export default {
  title: 'Devices',
  component: Devices,
} as ComponentMeta<typeof Devices>;

const Template: ComponentStory<typeof Devices> = args => <Devices {...args} />;

export const Default = Template.bind({});
