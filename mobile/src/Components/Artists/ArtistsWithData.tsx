import React, {FC} from 'react';
import Artists from './Artists';
import {useGetArtistsQuery} from '../../Hooks/GraphQL';
import {useNavigation} from '@react-navigation/native';

const ArtistsWithData: FC = () => {
  const navigation = useNavigation<any>();
  const {data, loading} = useGetArtistsQuery({
    variables: {
      offset: 0,
      limit: 10,
    },
  });
  const artists = !loading && data ? data.artists : [];

  const onPressArtist = (artist: any) => {
    navigation.navigate('ArtistDetails', {artist});
  };

  const onSeeAll = () => {
    navigation.navigate('Artists');
  };

  return (
    <Artists
      artists={artists.map(artist => ({
        id: artist.id,
        name: artist.name,
        cover: artist.picture,
      }))}
      onSeeAll={onSeeAll}
      onArtistPress={onPressArtist}
    />
  );
};

export default ArtistsWithData;
