import React, {FC} from 'react';
import {Album} from '../../Types';
import styled from '@emotion/native';
import Feather from 'react-native-vector-icons/Feather';
import {TouchableWithoutFeedback} from 'react-native';
import Ionicons from 'react-native-vector-icons/Ionicons';
import {useCover} from '../../Hooks/useCover';

const Container = styled.View`
  height: 80px;
  width: 100%;
  flex-direction: row;
  align-items: center;
  padding-left: 20px;
`;

const Cover = styled.Image`
  width: 60px;
  height: 60px;
`;

const NoAlbumCover = styled.View`
  width: 60px;
  height: 60px;
  background-color: #161515;
  align-items: center;
  justify-content: center;
`;

const Title = styled.Text`
  color: #fff;
  font-family: 'Gilroy-Bold';
  font-size: 16px;
`;

const Artist = styled.Text`
  color: #a7a7a9;
  font-family: 'Gilroy-Bold';
  font-size: 14px;
  margin-top: 2px;
`;

const AlbumInfo = styled.View`
  flex-direction: column;
  margin-left: 15px;
  flex: 1;
`;

const Button = styled.TouchableOpacity`
  height: 40px;
  width: 40px;
  align-items: center;
  justify-content: center;
`;

export type AlbumRowProps = {
  album: Album;
  onPressAlbum: () => void;
};

const AlbumRow: FC<AlbumRowProps> = props => {
  const {album, onPressAlbum} = props;
  const cover = useCover(album.cover);
  return (
    <TouchableWithoutFeedback onPress={onPressAlbum}>
      <Container>
        {album.cover && <Cover source={{uri: cover}} />}
        {!album.cover && (
          <NoAlbumCover>
            <Feather name="disc" size={40} color="#a7a7a9" />
          </NoAlbumCover>
        )}
        <AlbumInfo>
          <Title numberOfLines={1} ellipsizeMode="tail">
            {album.title}
          </Title>
          <Artist numberOfLines={1} ellipsizeMode="tail">
            {album.artist}
          </Artist>
        </AlbumInfo>
        <Button>
          <Ionicons name="ellipsis-vertical" color={'#ffffff99'} size={18} />
        </Button>
      </Container>
    </TouchableWithoutFeedback>
  );
};

export default AlbumRow;
