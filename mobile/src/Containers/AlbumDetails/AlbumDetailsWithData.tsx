import React, {FC} from 'react';
import {NativeStackScreenProps} from '@react-navigation/native-stack';
import AlbumDetails from './AlbumDetails';
import {MainStackParamList} from '../../Navigation/AppNavigation';
import {useNavigation} from '@react-navigation/native';
import {useGetAlbumQuery} from '../../Hooks/GraphQL';
import {Album} from '../../Types';
import {contextMenuState} from '../../Components/ContextMenu/ContextMenuState';
import {useRecoilState} from 'recoil';

type Props = NativeStackScreenProps<MainStackParamList, 'AlbumDetails'>;

const AlbumDetailsWithData: FC<Props> = ({route}) => {
  const navigation = useNavigation<any>();
  const [contextMenu, setContextMenu] = useRecoilState(contextMenuState);
  const {
    params: {album},
  } = route;
  const {data, loading} = useGetAlbumQuery({
    variables: {id: album.id},
    fetchPolicy: 'no-cache',
  });
  const tracks = !loading && data ? data.album.tracks : [];
  const onGoBack = () => navigation.goBack();
  const onContextMenu = (item: Album) => {
    setContextMenu({
      ...contextMenu,
      visible: true,
      type: 'album',
      enablePlayNext: true,
      item,
    });
  };
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
      onContextMenu={onContextMenu}
    />
  );
};

export default AlbumDetailsWithData;
