/**
 * @format
 */

import {AppRegistry} from 'react-native';
import App from './App';
import {name as appName} from './app.json';
import {ENABLE_STORYBOOK} from '@env';

console.log('ENABLE_STORYBOOK', ENABLE_STORYBOOK);

AppRegistry.registerComponent(appName, () => App);
