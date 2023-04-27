import styled from '@emotion/native';
import React, {FC} from 'react';
import {FlatList, TouchableWithoutFeedback} from 'react-native';
import Feather from 'react-native-vector-icons/Feather';
import {useCover} from '../../Hooks/useCover';

const Container = styled.View`
  width: 100%;
  margin-bottom: 50px;
`;

const Placeholder = styled.View`
  width: 100%;
  height: 228px;
  align-items: center;
  justify-content: center;
`;

const PlaceholderText = styled.Text`
  font-family: 'Gilroy-Bold';
  font-size: 16px;
  color: #a7a7a9;
`;

const Header = styled.View`
  margin: 0 20px;
  flex-direction: row;
  align-items: center;
  margin-bottom: 15px;
  justify-content: space-between;
`;

const Title = styled.Text`
  color: #fff;
  font-family: 'Gilroy-Bold';
  font-size: 20px;
`;

const NoAlbumCover = styled.View`
  width: 180px;
  height: 180px;
  background-color: #161515;
  margin-right: 8px;
  margin-left: 8px;
  border-radius: 3px;
  align-items: center;
  justify-content: center;
`;

const AlbumCover = styled.View`
  flex-direction: column;
`;

const Cover = styled.Image`
  width: 180px;
  height: 180px;
  margin-right: 8px;
  margin-left: 8px;
  border-radius: 3px;
`;

const AlbumTitle = styled.Text`
  color: #fff;
  font-family: 'Gilroy-Bold';
  font-size: 16px;
  width: 180px;
`;

const Artist = styled.Text`
  color: #a7a7a9;
  font-family: 'Gilroy-Bold';
  font-size: 14px;
  width: 180px;
`;

const AlbumInfo = styled.View`
  margin-left: 8px;
  margin-top: 10px;
`;

const SeeAll = styled.Text`
  color: #a7a7a9;
  font-family: 'Gilroy-Bold';
  font-size: 14px;
`;

const SeeAllContainer = styled.View`
  height: 30px;
  width: 60px;
  align-items: flex-end;
  justify-content: center;
`;

export type AlbumProps = {
  album: any;
  onPress: (album: any) => void;
};

export const Album: FC<AlbumProps> = ({album, onPress}) => {
  const cover = useCover(album.cover);
  return (
    <>
      {!album.cover && (
        <TouchableWithoutFeedback onPress={() => onPress(album)}>
          <AlbumCover>
            <NoAlbumCover>
              <Feather name="disc" size={100} color="#bdb9b958" />
            </NoAlbumCover>
            <AlbumInfo>
              <AlbumTitle ellipsizeMode="tail" numberOfLines={1}>
                {album.title}
              </AlbumTitle>
              <Artist ellipsizeMode="tail" numberOfLines={2}>
                {album.artist}
              </Artist>
            </AlbumInfo>
          </AlbumCover>
        </TouchableWithoutFeedback>
      )}
      {album.cover && (
        <TouchableWithoutFeedback onPress={() => onPress(album)}>
          <AlbumCover>
            <Cover
              source={{
                uri: cover,
              }}
            />
            <AlbumInfo>
              <AlbumTitle ellipsizeMode="tail" numberOfLines={1}>
                {album.title}
              </AlbumTitle>
              <Artist ellipsizeMode="tail" numberOfLines={2}>
                {album.artist}
              </Artist>
            </AlbumInfo>
          </AlbumCover>
        </TouchableWithoutFeedback>
      )}
    </>
  );
};

export type AlbumsProps = {
  albums: any;
  onSeeAll: () => void;
  onAlbumPress: (album: any) => void;
};

const Albums: FC<AlbumsProps> = props => {
  const {albums, onSeeAll, onAlbumPress} = props;
  return (
    <Container>
      <Header>
        <Title>Albums</Title>
        {albums.length > 0 && (
          <TouchableWithoutFeedback onPress={onSeeAll}>
            <SeeAllContainer>
              <SeeAll>See All</SeeAll>
            </SeeAllContainer>
          </TouchableWithoutFeedback>
        )}
      </Header>
      <>
        {albums.length > 0 && (
          <FlatList
            horizontal
            showsHorizontalScrollIndicator={false}
            data={albums}
            renderItem={({item}) => (
              <Album key={item.id} album={item} onPress={onAlbumPress} />
            )}
          />
        )}
        {albums.length === 0 && (
          <Placeholder>
            <PlaceholderText>No Albums</PlaceholderText>
          </Placeholder>
        )}
      </>
    </Container>
  );
};

export default Albums;
