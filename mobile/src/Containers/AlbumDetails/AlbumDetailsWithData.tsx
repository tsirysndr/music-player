import React, {FC} from 'react';
import {NativeStackScreenProps} from '@react-navigation/native-stack';
import AlbumDetails from './AlbumDetails';
import {MainStackParamList} from '../../Navigation/AppNavigation';
import {useNavigation} from '@react-navigation/native';

type Props = NativeStackScreenProps<MainStackParamList, 'AlbumDetails'>;

const AlbumDetailsWithData: FC<Props> = ({route}) => {
  const navigation = useNavigation<any>();
  const {
    params: {album},
  } = route;
  const onGoBack = () => navigation.goBack();
  return <AlbumDetails album={album} onGoBack={onGoBack} />;
};

export default AlbumDetailsWithData;
