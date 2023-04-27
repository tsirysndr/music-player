import React, {FC} from 'react';
import Albums from './Albums';
import {useGetAlbumsQuery} from '../../Hooks/GraphQL';
import {useNavigation} from '@react-navigation/native';

const AlbumsWithData: FC = () => {
  const navigation = useNavigation<any>();
  const {data, loading} = useGetAlbumsQuery({
    variables: {
      offset: 0,
      limit: 10,
    },
  });
  const albums = !loading && data ? data.albums : [];

  const onPressAlbum = (album: any) => {
    navigation.navigate('AlbumDetails', {album});
  };

  const onSeeAll = () => {
    navigation.navigate('Albums');
  };

  return (
    <Albums
      albums={albums.map(album => ({
        id: album.id,
        title: album.title,
        artist: album.artist,
        cover: album.cover,
      }))}
      onAlbumPress={onPressAlbum}
      onSeeAll={onSeeAll}
    />
  );
};

export default AlbumsWithData;
