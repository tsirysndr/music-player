import React, {FC} from 'react';
import Albums from './Albums';
import {useNavigation} from '@react-navigation/native';
import {useGetAlbumsQuery} from '../../Hooks/GraphQL';
import {Album} from '../../Types';

const AlbumsWithData: FC = () => {
  const navigation = useNavigation<any>();
  const {data, loading, fetchMore} = useGetAlbumsQuery({
    variables: {
      offset: 0,
      limit: 20,
    },
  });
  const albums = !loading && data ? data.albums : [];
  const onGoBack = () => navigation.goBack();

  const handleFetchMore = () => {
    fetchMore({
      variables: {
        offset: albums.length,
        limit: 20,
      },
      updateQuery: (prev, {fetchMoreResult}) => {
        if (!fetchMoreResult) {
          return prev;
        }
        return {
          albums: [...prev.albums, ...fetchMoreResult.albums],
        };
      },
    });
  };

  return (
    <Albums
      onGoBack={onGoBack}
      albums={albums.map(album => ({
        id: album.id,
        title: album.title,
        artist: album.artist,
        cover: album.cover!,
        year: album.year!,
      }))}
      fetchMore={handleFetchMore}
      onPressAlbum={(album: Album) =>
        navigation.navigate('AlbumDetails', {album})
      }
    />
  );
};

export default AlbumsWithData;
