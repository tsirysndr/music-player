import React from 'react';
import {ComponentMeta, ComponentStory} from '@storybook/react-native';
import AlbumDetails from './AlbumDetails';

export default {
  title: 'AlbumDetails',
  component: AlbumDetails,
} as ComponentMeta<typeof AlbumDetails>;

const Template: ComponentStory<typeof AlbumDetails> = args => (
  <AlbumDetails {...args} />
);

export const Default = Template.bind({});
