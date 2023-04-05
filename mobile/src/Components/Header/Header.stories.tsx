import React from 'react';
import Header from './Header';
import {ComponentMeta, ComponentStory} from '@storybook/react-native';

export default {
  title: 'Header',
  component: Header,
  argTypes: {
    onGoBack: {action: 'onGoBack'},
    onSearch: {action: 'onSearch'},
    onFilter: {action: 'onFilter'},
    title: {control: 'text'},
  },
} as ComponentMeta<typeof Header>;

const Template: ComponentStory<typeof Header> = args => <Header {...args} />;

export const Default = Template.bind({});

Default.args = {
  title: 'Albums',
};
