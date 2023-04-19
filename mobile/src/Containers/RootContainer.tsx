import React, {FC} from 'react';
import {View} from 'react-native';
import AppNavigator from '../Navigation/AppNavigation';
import {StyleSheet} from 'react-native';

const RootContainer: FC = () => (
  <View style={styles.container}>
    <AppNavigator />
  </View>
);

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
});

export default RootContainer;
