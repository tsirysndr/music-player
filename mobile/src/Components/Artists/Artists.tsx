import styled from '@emotion/native';
import React, {FC} from 'react';
import {FlatList, TouchableWithoutFeedback} from 'react-native';
import SvgMic from '../Icons/Mic';
import {Artist as ArtistType} from '../../Types';

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

const SeeAll = styled.Text`
  color: #a7a7a9;
  font-family: 'Gilroy-Bold';
  font-size: 14px;
`;

const ArtistCover = styled.View`
  flex-direction: column;
`;

const ArtistImage = styled.Image`
  width: 150px;
  height: 150px;
  margin-right: 8px;
  margin-left: 10px;
  border-radius: 90px;
`;

const NoArtistCover = styled.View`
  width: 150px;
  height: 150px;
  background-color: #161515;
  margin-right: 8px;
  margin-left: 10px;
  border-radius: 90px;
  align-items: center;
  justify-content: center;
`;

const ArtistName = styled.Text`
  color: #fff;
  font-family: 'Gilroy-Bold';
  font-size: 14px;
  margin-top: 10px;
  text-align: center;
  width: 150px;
  margin-right: 8px;
  margin-left: 10px;
`;

export type ArtistProps = {
  artist: ArtistType;
  onPress: (artist: ArtistType) => void;
};

const Artist: FC<ArtistProps> = props => {
  const {artist, onPress} = props;
  return (
    <TouchableWithoutFeedback onPress={() => onPress(artist)}>
      <ArtistCover>
        <>
          {artist.picture && <ArtistImage source={{uri: artist.picture}} />}
          {!artist.picture && (
            <NoArtistCover>
              <SvgMic />
            </NoArtistCover>
          )}
        </>
        <ArtistName ellipsizeMode="tail" numberOfLines={1}>
          {artist.name}
        </ArtistName>
      </ArtistCover>
    </TouchableWithoutFeedback>
  );
};

export type ArtistsProps = {
  artists: any;
  onSeeAll: () => void;
  onArtistPress: (artist: any) => void;
};

const Artists: FC<ArtistsProps> = props => {
  const {artists, onSeeAll, onArtistPress} = props;
  return (
    <Container>
      <Header>
        <Title>Artists</Title>
        <TouchableWithoutFeedback onPress={onSeeAll}>
          <SeeAll>See All</SeeAll>
        </TouchableWithoutFeedback>
      </Header>
      <>
        {artists.length === 0 && (
          <Placeholder>
            <PlaceholderText>No artists found</PlaceholderText>
          </Placeholder>
        )}
        {artists.length > 0 && (
          <FlatList
            horizontal
            showsHorizontalScrollIndicator={false}
            data={artists}
            renderItem={({item}) => (
              <Artist key={item.id} artist={item} onPress={onArtistPress} />
            )}
          />
        )}
      </>
    </Container>
  );
};

export default Artists;
