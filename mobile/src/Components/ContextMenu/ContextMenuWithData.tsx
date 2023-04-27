import React, {FC} from 'react';
import ContextMenu from './ContextMenu';
import {useRecoilState} from 'recoil';
import {contextMenuState} from './ContextMenuState';
import {Album, Track} from '../../Types';
import {useNavigation} from '@react-navigation/native';

const ContextMenuWithData: FC = () => {
  const navigation = useNavigation<any>();
  const [contextMenu, setContextMenu] = useRecoilState(contextMenuState);
  const onClose = () => {
    setContextMenu({
      ...contextMenu,
      visible: false,
    });
  };
  const onPlayNext = (item: Album | Track) => {
    onClose();
    console.log(item);
  };
  const onDownload = (item: Album | Track) => {
    onClose();
    console.log(item);
  };
  const onAddToPlaylist = (item: Album | Track) => {
    onClose();
    console.log(item);
  };
  const onGoToArtist = (item: Album | Track) => {
    onClose();
    switch (contextMenu.type) {
      case 'album':
        navigation.navigate('ArtistDetails', {artist: {}});
        break;
      case 'track':
        navigation.navigate('ArtistDetails', {artist: {}});
        break;
      default:
        break;
    }
  };
  const onGoToAlbum = (item: Album | Track) => {
    onClose();
    switch (contextMenu.type) {
      case 'album':
        navigation.navigate('AlbumDetails', {album: {}});
        break;
      case 'track':
        navigation.navigate('AlbumDetails', {album: {}});
        break;
      default:
        break;
    }
  };

  return (
    <ContextMenu
      isVisible={contextMenu.visible}
      onClose={onClose}
      item={contextMenu.item}
      type={contextMenu.type}
      enablePlayNext={contextMenu.enablePlayNext}
      onPlayNext={onPlayNext}
      onDownload={onDownload}
      onAddToPlaylist={onAddToPlaylist}
      onGoToArtist={onGoToArtist}
      onGoToAlbum={onGoToAlbum}
    />
  );
};

export default ContextMenuWithData;
