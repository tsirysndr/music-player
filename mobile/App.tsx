/**
 * Sample React Native App
 * https://github.com/facebook/react-native
 *
 * @format
 */
// import Storybook from './.storybook';

import React from 'react';
import {SafeAreaView, StyleSheet} from 'react-native';
import {RecoilRoot} from 'recoil';
import RootContainer from './src/Containers/RootContainer';

const App = () => {
  return (
    <RecoilRoot>
      <SafeAreaView style={styles.container}>
        <RootContainer />
      </SafeAreaView>
    </RecoilRoot>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#000',
  },
});

export default App;

//export default Storybook;
