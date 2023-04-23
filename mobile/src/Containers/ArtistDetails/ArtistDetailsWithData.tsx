import React, {FC} from 'react';
import ArtistDetails from './ArtistDetails';
import {useNavigation} from '@react-navigation/native';

const ArtistDetailsWithData: FC = () => {
  const navigation = useNavigation<any>();
  const onGoBack = () => navigation.goBack();
  return <ArtistDetails onGoBack={onGoBack} />;
};

export default ArtistDetailsWithData;
