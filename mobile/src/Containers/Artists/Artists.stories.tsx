import React from 'react';
import Artists from './Artists';
import {ComponentMeta, ComponentStory} from '@storybook/react-native';
import {artists} from '../../Mocks/Artists';

export default {
  title: 'ArtistsScreen',
  component: Artists,
} as ComponentMeta<typeof Artists>;

const Template: ComponentStory<typeof Artists> = args => <Artists {...args} />;

export const Default = Template.bind({});

Default.args = {
  artists,
};
