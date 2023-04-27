import React from 'react';
import {ComponentMeta, ComponentStory} from '@storybook/react-native';
import AlbumRow from './AlbumRow';
import {albums} from '../../Mocks/Albums';

export default {
  title: 'AlbumRow',
  component: AlbumRow,
  argTypes: {
    onPlay: {action: 'onPlay'},
  },
} as ComponentMeta<typeof AlbumRow>;

const Template: ComponentStory<typeof AlbumRow> = args => (
  <AlbumRow {...args} />
);

export const Default = Template.bind({});

Default.args = {
  album: albums[0],
};

export const NoCover = Template.bind({});

NoCover.args = {
  album: albums[2],
};
