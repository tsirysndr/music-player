import React, {FC} from 'react';
import Filter from './Filter';
import {MainStackParamList} from '../../Navigation/AppNavigation';
import {NativeStackScreenProps} from '@react-navigation/native-stack';
import {useNavigation} from '@react-navigation/native';
import {
  useGetAlbumsLazyQuery,
  useGetArtistsLazyQuery,
  useGetTracksLazyQuery,
} from '../../Hooks/GraphQL';

type Props = NativeStackScreenProps<MainStackParamList, 'Filter'>;

const FilterWithData: FC<Props> = ({route}) => {
  const {params} = route;
  const navigation = useNavigation<any>();
  const [getAlbums] = useGetAlbumsLazyQuery();
  const [getArtists] = useGetArtistsLazyQuery();
  const [getTracks] = useGetTracksLazyQuery();
  const placeholder: Record<string, string> = {
    track: 'Filter Tracks',
    album: 'Filter Albums',
    artist: 'Filter Artists',
  };
  const onSearch = (query: string) => {
    switch (params.type) {
      case 'track':
        getTracks({
          variables: {
            offset: 0,
            limit: 10,
            filter: query,
          },
        }).then(({data}) => {
          console.log(data);
        });
        break;
      case 'album':
        getAlbums({
          variables: {
            offset: 0,
            limit: 10,
            filter: query,
          },
        }).then(({data}) => {
          console.log(data);
        });
        break;
      case 'artist':
        getArtists({
          variables: {
            offset: 0,
            limit: 10,
            filter: query,
          },
        }).then(({data}) => {
          console.log(data);
        });
        break;
      default:
        break;
    }
  };
  return (
    <Filter
      onGoBack={() => navigation.goBack()}
      placeholder={placeholder[params.type]}
      onSearch={onSearch}
    />
  );
};

export default FilterWithData;
