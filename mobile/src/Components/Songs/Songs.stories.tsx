import React from 'react';
import {ComponentMeta, ComponentStory} from '@storybook/react-native';
import Songs from './Songs';

export default {
  title: 'Songs',
  component: Songs,
} as ComponentMeta<typeof Songs>;

const Template: ComponentStory<typeof Songs> = args => <Songs {...args} />;

export const Default = Template.bind({});
