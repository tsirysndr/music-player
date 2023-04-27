import React from 'react';
import {ComponentMeta, ComponentStory} from '@storybook/react-native';
import Artists from './Artists';
import {artists} from '../../Mocks/Artists';

export default {
  title: 'Artists',
  component: Artists,
  argTypes: {
    onArtistPress: {action: 'onArtistPress'},
    onSeeAll: {action: 'onSeeAll'},
  },
} as ComponentMeta<typeof Artists>;

const Template: ComponentStory<typeof Artists> = args => <Artists {...args} />;

export const Default = Template.bind({});

Default.args = {
  artists,
};
