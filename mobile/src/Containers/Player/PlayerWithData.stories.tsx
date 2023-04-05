import React from 'react';
import {ComponentMeta, ComponentStory} from '@storybook/react-native';
import Player from './PlayerWithData';

export default {
  title: 'PlayerWithData',
  component: Player,
} as ComponentMeta<typeof Player>;

const Template: ComponentStory<typeof Player> = args => <Player {...args} />;

export const Default = Template.bind({});

Default.args = {};
