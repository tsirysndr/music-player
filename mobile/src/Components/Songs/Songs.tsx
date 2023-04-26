import styled from '@emotion/native';
import React, {FC} from 'react';
import {TouchableWithoutFeedback} from 'react-native';
import {Track as TrackType} from '../../Types';
import TrackRow from '../TrackRow';

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

const SeeAllContainer = styled.View`
  height: 30px;
  width: 60px;
  align-items: flex-end;
  justify-content: center;
`;

export type SongsProps = {
  tracks: TrackType[];
  onSeeAll: () => void;
};

const Songs: FC<SongsProps> = props => {
  const {tracks, onSeeAll} = props;
  return (
    <Container>
      <Header>
        <Title>Tracks</Title>
        <TouchableWithoutFeedback onPress={onSeeAll}>
          <SeeAllContainer>
            <SeeAll>See All</SeeAll>
          </SeeAllContainer>
        </TouchableWithoutFeedback>
      </Header>
      {tracks.map((item: any) => (
        <TrackRow key={item.id} track={item} />
      ))}
    </Container>
  );
};

export default Songs;
