import {createStackNavigator} from '@react-navigation/stack';
import {createBottomTabNavigator} from '@react-navigation/bottom-tabs';
import React, {FC} from 'react';
import Home from '../Containers/Home';
import {NavigationContainer} from '@react-navigation/native';
import Player from '../Containers/Player';
import MaterialIcons from 'react-native-vector-icons/MaterialIcons';
import Feather from 'react-native-vector-icons/Feather';
import IonicIcons from 'react-native-vector-icons/Ionicons';
import AntDesign from 'react-native-vector-icons/AntDesign';

const RootStack = createStackNavigator();
const MainStack = createStackNavigator();
const Tab = createBottomTabNavigator();

const TabBarHomeIcon: FC<{color: string}> = ({color}) => (
  <IonicIcons name="headset-outline" size={24} color={color} />
);

const TabBarSearchIcon: FC<{color: string}> = ({color}) => (
  <Feather name="search" size={24} color={color} />
);

const TabBarDevicesIcon: FC<{color: string}> = ({color}) => (
  <MaterialIcons name="devices" size={24} color={color} />
);

const TabBarAccountIcon: FC<{color: string}> = ({color}) => (
  <AntDesign name="user" size={24} color={color} />
);

const Tabs: FC = () => (
  <Tab.Navigator
    screenOptions={{
      tabBarHideOnKeyboard: true,
      tabBarActiveTintColor: '#8900eb',
      tabBarStyle: {backgroundColor: '#000', borderTopWidth: 0, elevation: 0},
      // tabBarShowLabel: false,
    }}>
    <Tab.Screen
      name="Library"
      component={Home}
      options={{
        headerShown: false,
        tabBarIcon: TabBarHomeIcon,
      }}
    />
    <Tab.Screen
      name="Search"
      component={Home}
      options={{
        headerShown: false,
        tabBarIcon: TabBarSearchIcon,
      }}
    />
    <Tab.Screen
      name="Devices"
      component={Home}
      options={{
        headerShown: false,
        tabBarIcon: TabBarDevicesIcon,
      }}
    />
    <Tab.Screen
      name="Account"
      component={Home}
      options={{
        headerShown: false,
        tabBarIcon: TabBarAccountIcon,
      }}
    />
  </Tab.Navigator>
);

const MainNavigator: FC = () => (
  <MainStack.Navigator screenOptions={{headerShown: false}}>
    <MainStack.Screen name="Tabs" component={Tabs} />
  </MainStack.Navigator>
);

const AppNavigator: FC = () => (
  <NavigationContainer>
    <RootStack.Navigator>
      <RootStack.Group
        screenOptions={{
          headerShown: false,
          gestureEnabled: true,
          headerTransparent: true,
          cardStyle: {backgroundColor: 'transparent'},
        }}>
        <RootStack.Screen name="Main" component={MainNavigator} />
      </RootStack.Group>
      <RootStack.Group screenOptions={{presentation: 'modal'}}>
        <RootStack.Screen name="Player" component={Player} />
      </RootStack.Group>
    </RootStack.Navigator>
  </NavigationContainer>
);

export default AppNavigator;
