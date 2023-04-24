import React, {FC} from 'react';
import ArtistDetails from './ArtistDetails';
import {useNavigation} from '@react-navigation/native';
import {MainStackParamList} from '../../Navigation/AppNavigation';
import {NativeStackScreenProps} from '@react-navigation/native-stack';

type Props = NativeStackScreenProps<MainStackParamList, 'ArtistDetails'>;

const ArtistDetailsWithData: FC<Props> = ({route}) => {
  const navigation = useNavigation<any>();
  const {
    params: {artist},
  } = route;
  console.log(artist);
  const onGoBack = () => navigation.goBack();
  return <ArtistDetails onGoBack={onGoBack} artist={artist} />;
};

export default ArtistDetailsWithData;
