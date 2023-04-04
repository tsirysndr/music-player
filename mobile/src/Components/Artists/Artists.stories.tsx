import React from 'react';
import {ComponentMeta, ComponentStory} from '@storybook/react-native';
import Artists from './Artists';

export default {
  title: 'Artists',
  component: Artists,
} as ComponentMeta<typeof Artists>;

const Template: ComponentStory<typeof Artists> = args => <Artists {...args} />;

export const Default = Template.bind({});
