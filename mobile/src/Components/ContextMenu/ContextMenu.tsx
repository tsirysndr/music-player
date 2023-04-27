import React, {FC} from 'react';
import {StyleSheet} from 'react-native';
import Modal from 'react-native-modal';
import styled from '@emotion/native';
import Feather from 'react-native-vector-icons/Feather';
import MaterialCommunity from 'react-native-vector-icons/MaterialCommunityIcons';
import MaterialIcons from 'react-native-vector-icons/MaterialIcons';
import {Album, Artist, Track} from '../../Types';
import {useCover} from '../../Hooks/useCover';
import SvgMic from '../Icons/Mic';

const Container = styled.View`
  background-color: #000;
  height: 420px;
`;

const MetadataRow = styled.View`
  height: 110px;
  width: 100%;
  flex-direction: row;
  align-items: center;
  padding-left: 20px;
  background-color: #000;
`;

const Separator = styled.View`
  height: 1px;
  width: 100%;
  background-color: #3e3e3ec4;
  margin-bottom: 10px;
`;

const Cover = styled.Image`
  width: 80px;
  height: 80px;
`;

const NoAlbumCover = styled.View`
  width: 80px;
  height: 80px;
  background-color: #161515;
  align-items: center;
  justify-content: center;
`;

const Title = styled.Text`
  color: #fff;
  font-family: 'Gilroy-Bold';
  font-size: 16px;
`;

const ArtistName = styled.Text`
  color: #a7a7a9;
  font-family: 'Gilroy-Bold';
  font-size: 14px;
  margin-top: 2px;
`;

const MediaInfo = styled.View`
  flex-direction: column;
  margin-left: 15px;
  flex: 1;
`;

const IconWrapper = styled.View`
  height: 40px;
  width: 40px;
  justify-content: center;
  margin-right: 10px;
`;

const Action = styled.TouchableWithoutFeedback``;

const ActionWrapper = styled.View`
  height: 60px;
  width: 100%;
  flex-direction: row;
  align-items: center;
  padding-left: 20px;
  background-color: #000;
`;

const ActionTitle = styled.Text`
  color: #fff;
  font-family: 'Gilroy-Bold';
  font-size: 16px;
`;

export type ContextMenuProps = {
  isVisible: boolean;
  onClose: () => void;
  item?: Artist | Album | Track;
  type: 'artist' | 'album' | 'track' | '';
  enablePlayNext?: boolean;
  onPlayNext: (item: Album | Track) => void;
  onAddToPlaylist: (item: Album | Track) => void;
  onDownload: (item: Album | Track) => void;
  onGoToArtist: (item: Album | Track) => void;
  onGoToAlbum: (item: Album | Track) => void;
};

const ContextMenu: FC<ContextMenuProps> = props => {
  const {
    isVisible,
    onClose,
    onAddToPlaylist,
    onDownload,
    onGoToAlbum,
    onGoToArtist,
    onPlayNext,
    item,
    type,
    enablePlayNext,
  } = props;
  const itemCover = {
    artist: (item as Artist)?.picture,
    album: (item as Album)?.cover,
    track: (item as Track)?.cover,
    '': undefined,
  };
  const cover = useCover(itemCover[type]);
  return (
    <Modal
      isVisible={isVisible}
      backdropOpacity={0.03}
      onBackdropPress={onClose}
      onSwipeComplete={onClose}
      swipeDirection={['down']}
      style={styles.modal}>
      <Container>
        <MetadataRow>
          {itemCover[type] && <Cover source={{uri: cover}} />}
          {!itemCover[type] && (
            <NoAlbumCover>
              <Feather name="disc" size={40} color="#a7a7a9" />
            </NoAlbumCover>
          )}
          <MediaInfo>
            <Title numberOfLines={1} ellipsizeMode="tail">
              {type === 'artist'
                ? (item as Artist)?.name
                : (item as Album)?.title}
            </Title>
            <ArtistName numberOfLines={1} ellipsizeMode="tail">
              {type === 'artist'
                ? (item as Artist)?.name
                : (item as Album)?.artist}
            </ArtistName>
          </MediaInfo>
        </MetadataRow>
        <Separator />
        {enablePlayNext && (
          <Action onPress={() => onPlayNext(item as Album | Track)}>
            <ActionWrapper>
              <IconWrapper>
                <MaterialIcons name="playlist-play" size={31} color="#ab28fc" />
              </IconWrapper>
              <ActionTitle>Play next</ActionTitle>
            </ActionWrapper>
          </Action>
        )}
        {(type === 'track' || type === 'album') && (
          <>
            <Action onPress={() => onDownload(item as Album | Track)}>
              <ActionWrapper>
                <IconWrapper>
                  <MaterialCommunity
                    name="download"
                    size={25}
                    color="#ab28fc"
                  />
                </IconWrapper>
                <ActionTitle>Download</ActionTitle>
              </ActionWrapper>
            </Action>
            <Action onPress={() => onAddToPlaylist(item as Album | Track)}>
              <ActionWrapper>
                <IconWrapper>
                  <MaterialIcons
                    name="playlist-add"
                    size={28}
                    color="#ab28fc"
                  />
                </IconWrapper>
                <ActionTitle>Add to playlist</ActionTitle>
              </ActionWrapper>
            </Action>
            {type !== 'album' && (
              <Action onPress={() => onGoToAlbum(item as Album | Track)}>
                <ActionWrapper>
                  <IconWrapper>
                    <Feather name="disc" size={24} color="#ab28fc" />
                  </IconWrapper>
                  <ActionTitle>Go to Album</ActionTitle>
                </ActionWrapper>
              </Action>
            )}
            <Action onPress={() => onGoToArtist(item as Album | Track)}>
              <ActionWrapper>
                <IconWrapper>
                  <SvgMic height={26} width={26} fill="#ab28fc" />
                </IconWrapper>
                <ActionTitle>Go to Artist</ActionTitle>
              </ActionWrapper>
            </Action>
          </>
        )}
      </Container>
    </Modal>
  );
};

const styles = StyleSheet.create({
  modal: {
    justifyContent: 'flex-end',
    margin: 0,
  },
});

export default ContextMenu;
