import React from 'react';
import {ComponentMeta, ComponentStory} from '@storybook/react-native';
import ArtistRow from './ArtistRow';
import {artists} from '../../Mocks/Artists';

export default {
  title: 'ArtistRow',
  component: ArtistRow,
} as ComponentMeta<typeof ArtistRow>;

const Template: ComponentStory<typeof ArtistRow> = args => (
  <ArtistRow {...args} />
);

export const Default = Template.bind({});

Default.args = {
  artist: artists[0],
};
