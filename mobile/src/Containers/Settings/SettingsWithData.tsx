import React, {FC} from 'react';
import Settings from './Settings';
import {useNavigation} from '@react-navigation/native';

const SettingsWithData: FC = () => {
  const navigation = useNavigation<any>();
  return <Settings onGoBack={() => navigation.goBack()} />;
};

export default SettingsWithData;
