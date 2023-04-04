import styled from '@emotion/native';
import React, {FC} from 'react';
import {FlatList, TouchableWithoutFeedback, View} from 'react-native';
import Feather from 'react-native-vector-icons/Feather';

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

export type AlbumProps = {
  album: any;
  onPress: (album: any) => void;
};

const Album: FC<AlbumProps> = ({album, onPress}) => (
  <>
    {false && (
      <TouchableWithoutFeedback onPress={() => onPress(album)}>
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
      </TouchableWithoutFeedback>
    )}
    {true && (
      <TouchableWithoutFeedback onPress={() => onPress(album)}>
        <AlbumCover>
          <Cover
            source={{
              uri: album.cover,
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

export type AlbumsProps = {
  albums: any;
  onSeeAll: () => void;
  onAlbumPress: (album: any) => void;
};

const Albums: FC<AlbumsProps> = props => {
  const {albums, onSeeAll, onAlbumPress} = props;
  return (
    <View>
      <Header>
        <Title>Albums</Title>
        <TouchableWithoutFeedback onPress={onSeeAll}>
          <SeeAll>See All</SeeAll>
        </TouchableWithoutFeedback>
      </Header>
      <FlatList
        horizontal
        showsHorizontalScrollIndicator={false}
        data={albums}
        renderItem={({item}) => (
          <Album key={item.id} album={item} onPress={onAlbumPress} />
        )}
      />
    </View>
  );
};

export default Albums;
