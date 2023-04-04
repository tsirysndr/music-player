import React from 'react';
import {ComponentMeta, ComponentStory} from '@storybook/react-native';
import Albums from './Albums';
import {albums} from '../../Mocks/Albums';

export default {
  title: 'Albums',
  component: Albums,
  argTypes: {
    onAlbumPress: {action: 'onAlbumPress'},
    onSeeAll: {action: 'onSeeAll'},
  },
} as ComponentMeta<typeof Albums>;

const Template: ComponentStory<typeof Albums> = args => <Albums {...args} />;

export const Default = Template.bind({});

Default.args = {
  albums,
};

export const Empty = Template.bind({});

Empty.args = {
  albums: [],
};
