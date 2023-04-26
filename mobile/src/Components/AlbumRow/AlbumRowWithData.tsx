import React, {FC} from 'react';
import AlbumRow from './AlbumRow';
import {Album} from '../../Types';
import {useNavigation} from '@react-navigation/native';
import {useRecoilState} from 'recoil';
import {contextMenuState} from '../ContextMenu/ContextMenuState';

export type AlbumRowWithDataProps = {
  album: Album;
};

const AlbumRowWithData: FC<AlbumRowWithDataProps> = ({album}) => {
  const navigation = useNavigation<any>();
  const [contextMenu, setContextMenu] = useRecoilState(contextMenuState);
  const onPressAlbum = () => {
    navigation.navigate('AlbumDetails', {album});
  };
  const onContextMenu = (item: Album) => {
    setContextMenu({
      ...contextMenu,
      visible: true,
      type: 'album',
      item,
    });
  };

  return (
    <AlbumRow
      album={album}
      onPressAlbum={onPressAlbum}
      onPressContextMenu={onContextMenu}
    />
  );
};

export default AlbumRowWithData;
