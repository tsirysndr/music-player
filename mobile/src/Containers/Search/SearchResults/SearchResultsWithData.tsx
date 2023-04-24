import React, {FC} from 'react';
import SearchResults from './SearchResults';
import {useRecoilValue} from 'recoil';
import {searchState} from '../SearchState';
import {useNavigation} from '@react-navigation/native';

const SearchWithData: FC = () => {
  const navigation = useNavigation<any>();
  const props = useRecoilValue(searchState);
  const onPressAlbum = (album: any) => {
    navigation.navigate('Album', {album});
  };
  const onPressArtist = (artist: any) => {
    navigation.navigate('Artist', {artist});
  };
  return (
    <SearchResults
      {...props}
      onPressAlbum={onPressAlbum}
      onPressArtist={onPressArtist}
    />
  );
};

export default SearchWithData;
