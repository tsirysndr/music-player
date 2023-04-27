import {createStackNavigator} from '@react-navigation/stack';
import {createBottomTabNavigator} from '@react-navigation/bottom-tabs';
import React, {FC} from 'react';
import Home from '../Containers/Home';
import {NavigationContainer, DarkTheme} from '@react-navigation/native';
import Player from '../Containers/Player';
import MaterialIcons from 'react-native-vector-icons/MaterialIcons';
import Feather from 'react-native-vector-icons/Feather';
import IonicIcons from 'react-native-vector-icons/Ionicons';
import AntDesign from 'react-native-vector-icons/AntDesign';
import Search from '../Containers/Search';
import Account from '../Containers/Account';
import Devices from '../Containers/Devices';
import Albums from '../Containers/Albums';
import Artists from '../Containers/Artists';
import Tracks from '../Containers/Tracks';
import ArtistDetails from '../Containers/ArtistDetails';
import AlbumDetails from '../Containers/AlbumDetails';
import Settings from '../Containers/Settings';
import Filter from '../Containers/Filter';

const RootStack = createStackNavigator();
const MainStack = createStackNavigator();
const HomeStack = createStackNavigator();
const SearchStack = createStackNavigator();
const Tab = createBottomTabNavigator();

export type MainStackParamList = {
  Home: undefined;
  Search: undefined;
  Devices: undefined;
  Account: undefined;
  Albums: {albums?: any[]};
  Artists: undefined;
  Tracks: {tracks?: any[]};
  ArtistDetails: {artist: any};
  AlbumDetails: {album: any};
  Filter: {type: string; tracks?: any[]; albums?: any[]};
};

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
      component={HomeStackScreen}
      options={{
        headerShown: false,
        tabBarIcon: TabBarHomeIcon,
      }}
    />
    <Tab.Screen
      name="Search"
      component={SearchStackScreen}
      options={{
        headerShown: false,
        tabBarIcon: TabBarSearchIcon,
      }}
    />
    <Tab.Screen
      name="Devices"
      component={Devices}
      options={{
        headerShown: false,
        tabBarIcon: TabBarDevicesIcon,
      }}
    />
    <Tab.Screen
      name="Account"
      component={Account}
      options={{
        headerShown: false,
        tabBarIcon: TabBarAccountIcon,
      }}
    />
  </Tab.Navigator>
);

const HomeStackScreen: FC = () => (
  <HomeStack.Navigator screenOptions={{headerShown: false}}>
    <HomeStack.Screen name="Home" component={Home} />
    <HomeStack.Screen name="Albums" component={Albums as any} />
    <HomeStack.Screen name="Artists" component={Artists} />
    <HomeStack.Screen name="Tracks" component={Tracks as any} />
    <HomeStack.Screen name="ArtistDetails" component={ArtistDetails as any} />
    <HomeStack.Screen name="AlbumDetails" component={AlbumDetails as any} />
    <HomeStack.Screen name="Filter" component={Filter as any} />
    <HomeStack.Screen name="Settings" component={Settings as any} />
  </HomeStack.Navigator>
);

const SearchStackScreen: FC = () => (
  <SearchStack.Navigator screenOptions={{headerShown: false}}>
    <SearchStack.Screen name="SearchScreen" component={Search} />
    <SearchStack.Screen name="Albums" component={Albums as any} />
    <SearchStack.Screen name="Artists" component={Artists} />
    <SearchStack.Screen name="Tracks" component={Tracks as any} />
    <SearchStack.Screen name="ArtistDetails" component={ArtistDetails as any} />
    <SearchStack.Screen name="AlbumDetails" component={AlbumDetails as any} />
    <SearchStack.Screen name="Filter" component={Filter as any} />
  </SearchStack.Navigator>
);

const MainNavigator: FC = () => (
  <MainStack.Navigator screenOptions={{headerShown: false}}>
    <MainStack.Screen name="Tabs" component={Tabs} />
  </MainStack.Navigator>
);

const AppNavigator: FC = () => (
  <NavigationContainer theme={DarkTheme}>
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
      <RootStack.Group
        screenOptions={{
          headerShown: false,
          gestureEnabled: true,
          animationEnabled: true,
          presentation: 'modal',
        }}>
        <RootStack.Screen name="Player" component={Player} />
      </RootStack.Group>
    </RootStack.Navigator>
  </NavigationContainer>
);

export default AppNavigator;
