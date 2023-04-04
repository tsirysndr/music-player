import React from 'react';
import Search from './Search';
import {ComponentMeta, ComponentStory} from '@storybook/react-native';

export default {
  title: 'Search',
  component: Search,
} as ComponentMeta<typeof Search>;

const Template: ComponentStory<typeof Search> = args => <Search {...args} />;

export const Default = Template.bind({});
