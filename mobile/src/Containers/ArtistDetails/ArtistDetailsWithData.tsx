import React, {FC} from 'react';
import ArtistDetails from './ArtistDetails';
import {useNavigation} from '@react-navigation/native';
import {MainStackParamList} from '../../Navigation/AppNavigation';
import {NativeStackScreenProps} from '@react-navigation/native-stack';
import {useGetArtistQuery} from '../../Hooks/GraphQL';

type Props = NativeStackScreenProps<MainStackParamList, 'ArtistDetails'>;

const ArtistDetailsWithData: FC<Props> = ({route}) => {
  const navigation = useNavigation<any>();
  const {
    params: {artist},
  } = route;
  console.log(artist);

  const {data, loading} = useGetArtistQuery({
    variables: {
      id: artist.id,
    },
  });

  const albums = !loading && data ? data.artist.albums : [];
  const tracks = !loading && data ? data.artist.songs : [];
  const onGoBack = () => navigation.goBack();
  const onPressAlbum = (album: any) =>
    navigation.navigate('AlbumDetails', {album});

  return (
    <ArtistDetails
      onGoBack={onGoBack}
      onPressAlbum={onPressAlbum}
      artist={{
        ...artist,
        albums: albums.map(album => ({
          id: album.id,
          title: album.title,
          artist: album.artist,
          cover: album.cover!,
          year: album.year!,
        })),
        tracks: tracks.map(track => ({
          id: track.id,
          title: track.title,
          artist: track.artist,
          album: track.album.title,
          duration: track.duration!,
          cover: track.album.cover || undefined,
          artistId: track.artists[0].id,
          albumId: track.album.id,
        })),
      }}
    />
  );
};

export default ArtistDetailsWithData;
