import React, {FC} from 'react';
import {NativeStackScreenProps} from '@react-navigation/native-stack';
import AlbumDetails from './AlbumDetails';
import {MainStackParamList} from '../../Navigation/AppNavigation';
import {useNavigation} from '@react-navigation/native';
import {useGetAlbumQuery} from '../../Hooks/GraphQL';

type Props = NativeStackScreenProps<MainStackParamList, 'AlbumDetails'>;

const AlbumDetailsWithData: FC<Props> = ({route}) => {
  const navigation = useNavigation<any>();
  const {
    params: {album},
  } = route;
  const {data, loading} = useGetAlbumQuery({variables: {id: album.id}});
  const tracks = !loading && data ? data.album.tracks : [];
  const onGoBack = () => navigation.goBack();
  return (
    <AlbumDetails
      album={{
        ...album,
        tracks: tracks.map(track => ({
          ...track,
          cover: album.cover,
        })),
      }}
      onGoBack={onGoBack}
    />
  );
};

export default AlbumDetailsWithData;
