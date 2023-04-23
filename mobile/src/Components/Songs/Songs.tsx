import styled from '@emotion/native';
import React, {FC} from 'react';
import {TouchableWithoutFeedback} from 'react-native';
import Feather from 'react-native-vector-icons/Feather';
import Ionicons from 'react-native-vector-icons/Ionicons';
import {Track as TrackType} from '../../Types';
import {useCover} from '../../Hooks/useCover';

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
  margin-left: 20px;
  height: 60px;
`;

const TrackTitle = styled.Text<{active?: boolean}>`
  color: #fff;
  font-family: 'Gilroy-Bold';
  font-size: 16px;
  ${props => props.active && 'color: #ab28fc;'}
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
  flex: 1;
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

const TrackWrapper = styled.View`
  background-color: #232323;
`;

const TouchableTrack = styled.TouchableOpacity`
  background-color: #000;
  justify-content: center;
  height: 70px;
`;

const Button = styled.TouchableOpacity`
  height: 40px;
  width: 40px;
  align-items: center;
  justify-content: center;
  z-index: 1;
`;

export type TrackProps = {
  item: any;
  active: boolean;
  onPress: (item: any) => void;
};

const Track: FC<TrackProps> = props => {
  const {item, active, onPress} = props;
  const cover = useCover(item.cover);
  return (
    <TrackWrapper>
      <TouchableTrack onPress={() => onPress(item)}>
        <TrackRow>
          {item.cover && <Cover source={{uri: cover}} />}
          {!item.cover && (
            <NoCover>
              <Feather name="music" size={30} color="#a7a7a9" />
            </NoCover>
          )}
          <TrackInfo>
            <TrackTitle active={active} ellipsizeMode="tail" numberOfLines={1}>
              {item.title}
            </TrackTitle>
            <TrackArtist ellipsizeMode="tail" numberOfLines={1}>
              {item.artist}
            </TrackArtist>
          </TrackInfo>
          <Button>
            <Ionicons name="ellipsis-vertical" color={'#ffffff99'} size={18} />
          </Button>
        </TrackRow>
      </TouchableTrack>
    </TrackWrapper>
  );
};

export type SongsProps = {
  tracks: TrackType[];
  currentTrack?: TrackType;
  onSeeAll: () => void;
  onPressTrack: (item: any) => void;
};

const Songs: FC<SongsProps> = props => {
  const {tracks, currentTrack, onSeeAll, onPressTrack} = props;
  return (
    <Container>
      <Header>
        <Title>Tracks</Title>
        <TouchableWithoutFeedback onPress={onSeeAll}>
          <SeeAll>See All</SeeAll>
        </TouchableWithoutFeedback>
      </Header>
      {tracks.map((item: any) => (
        <Track
          key={item.id}
          item={item}
          active={item.id === currentTrack?.id}
          onPress={onPressTrack}
        />
      ))}
    </Container>
  );
};

export default Songs;
