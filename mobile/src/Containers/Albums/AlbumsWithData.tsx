import React, {FC} from 'react';
import Albums from './Albums';
import {useNavigation} from '@react-navigation/native';
import {useGetAlbumsQuery} from '../../Hooks/GraphQL';
import {Album} from '../../Types';
import {MainStackParamList} from '../../Navigation/AppNavigation';
import {NativeStackScreenProps} from '@react-navigation/native-stack';

type Props = NativeStackScreenProps<MainStackParamList, 'Albums'>;

const AlbumsWithData: FC<Props> = ({route}) => {
  const navigation = useNavigation<any>();
  const {params} = route;
  const {data, loading, fetchMore} = useGetAlbumsQuery({
    variables: {
      offset: 0,
      limit: 20,
    },
  });
  const albums = params?.albums || (!loading && data ? data.albums : []);
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
      fetchMore={params?.albums ? () => {} : handleFetchMore}
      onPressAlbum={(album: Album) =>
        navigation.navigate('AlbumDetails', {album})
      }
    />
  );
};

export default AlbumsWithData;
