import styled from '@emotion/native';
import React, {FC} from 'react';
import {TouchableOpacity, TouchableWithoutFeedback} from 'react-native';
import Feather from 'react-native-vector-icons/Feather';

const Container = styled.View`
  width: 100%;
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
  font-size: 24px;
`;

const SeeAll = styled.Text`
  color: #a7a7a9;
  font-family: 'Gilroy-Bold';
  font-size: 14px;
`;

const TrackRow = styled.View`
  flex-direction: row;
  align-items: center;
  margin: 0 20px;
  height: 60px;
  margin-bottom: 15px;
`;

const TrackTitle = styled.Text`
  color: #fff;
  font-family: 'Gilroy-Bold';
  font-size: 16px;
`;

const TrackArtist = styled.Text`
  color: #a7a7a9;
  font-family: 'Gilroy-Bold';
  font-size: 14px;
  margin-top: 2px;
`;

const TrackInfo = styled.View`
  flex-direction: column;
  margin-left: 20px;
`;

const Cover = styled.Image`
  width: 60px;
  height: 60px;
  border-radius: 3px;
`;

const NoCover = styled.View`
  width: 60px;
  height: 60px;
  background-color: #161515;
  align-items: center;
  justify-content: center;
  border-radius: 3px;
`;

export type TrackProps = {
  item: any;
  onPress: (item: any) => void;
};

const Track: FC<TrackProps> = props => {
  const {item, onPress} = props;
  return (
    <TouchableOpacity onPress={() => onPress(item)}>
      <TrackRow>
        {item.cover && <Cover source={{uri: item.cover}} />}
        {!item.cover && (
          <NoCover>
            <Feather name="music" size={30} color="#a7a7a9" />
          </NoCover>
        )}
        <TrackInfo>
          <TrackTitle ellipsizeMode="tail" numberOfLines={1}>
            {item.title}
          </TrackTitle>
          <TrackArtist ellipsizeMode="tail" numberOfLines={1}>
            {item.artist}
          </TrackArtist>
        </TrackInfo>
      </TrackRow>
    </TouchableOpacity>
  );
};

export type SongsProps = {
  tracks: any;
  onSeeAll: () => void;
  onPressTrack: (item: any) => void;
};

const Songs: FC<SongsProps> = props => {
  const {tracks, onSeeAll, onPressTrack} = props;
  return (
    <Container>
      <Header>
        <Title>Tracks</Title>
        <TouchableWithoutFeedback onPress={onSeeAll}>
          <SeeAll>See All</SeeAll>
        </TouchableWithoutFeedback>
      </Header>
      {tracks.map((item: any) => (
        <Track key={item.id} item={item} onPress={onPressTrack} />
      ))}
    </Container>
  );
};

export default Songs;
