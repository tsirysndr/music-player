import React, {FC} from 'react';
import Artists from './Artists';
import {useNavigation} from '@react-navigation/native';
import {useGetArtistsQuery} from '../../Hooks/GraphQL';

const ArtistsWithData: FC = () => {
  const navigation = useNavigation<any>();
  const {data, loading, fetchMore} = useGetArtistsQuery({
    variables: {
      offset: 0,
      limit: 100,
    },
  });
  const artists = !loading && data ? data.artists : [];
  const onGoBack = () => navigation.goBack();

  const handleFetchMore = () => {
    fetchMore({
      variables: {
        offset: artists.length,
        limit: 100,
      },
      updateQuery: (prev, {fetchMoreResult}) => {
        if (!fetchMoreResult) {
          return prev;
        }
        return {
          artists: [...prev.artists, ...fetchMoreResult.artists],
        };
      },
    });
  };

  const onPressArtist = (artist: any) =>
    navigation.navigate('ArtistDetails', {artist});

  return (
    <Artists
      onGoBack={onGoBack}
      artists={artists.map(artist => ({
        id: artist.id,
        name: artist.name,
        cover: artist.picture,
      }))}
      fetchMore={handleFetchMore}
      onPressArtist={onPressArtist}
    />
  );
};

export default ArtistsWithData;
