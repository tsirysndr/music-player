import React from 'react';
import {ComponentMeta, ComponentStory} from '@storybook/react-native';
import Albums from './Albums';
import {albums} from '../../Mocks/Albums';

export default {
  title: 'AlbumsScreen',
  component: Albums,
  argTypes: {
    onGoBack: {action: 'onGoBack'},
    fetchMore: {action: 'fetchMore'},
  },
} as ComponentMeta<typeof Albums>;

const Template: ComponentStory<typeof Albums> = args => <Albums {...args} />;

export const Default = Template.bind({});

Default.args = {
  albums,
};
