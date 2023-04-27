import React from 'react';
import {View} from 'react-native';
import {MyButton} from './Button';

const MyButtonMeta = {
  title: 'Button',
  component: MyButton,
  argTypes: {
    onPress: {action: 'pressed the button'},
  },
  args: {
    text: 'Hello world',
  },
  decorators: [
    Story => (
      // eslint-disable-next-line react-native/no-inline-styles
      <View style={{alignItems: 'center', justifyContent: 'center', flex: 1}}>
        <Story />
      </View>
    ),
  ],
};

export default MyButtonMeta;

export const Basic = {};
