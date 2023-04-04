import React from 'react';
import {ComponentMeta, ComponentStory} from '@storybook/react-native';
import Account from './Account';

export default {
  title: 'Account',
  component: Account,
} as ComponentMeta<typeof Account>;

const Template: ComponentStory<typeof Account> = args => <Account {...args} />;

export const Default = Template.bind({});
