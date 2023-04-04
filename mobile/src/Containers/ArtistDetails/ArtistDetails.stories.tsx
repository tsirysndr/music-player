import React from 'react';
import {ComponentMeta, ComponentStory} from '@storybook/react-native';
import ArtistDetails from './ArtistDetails';

export default {
  title: 'ArtistDetails',
  component: ArtistDetails,
} as ComponentMeta<typeof ArtistDetails>;

const Template: ComponentStory<typeof ArtistDetails> = args => (
  <ArtistDetails {...args} />
);

export const Default = Template.bind({});
