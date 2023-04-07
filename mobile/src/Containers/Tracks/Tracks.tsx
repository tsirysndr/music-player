import React, {FC} from 'react';
import Header from '../../Components/Header';
import styled from '@emotion/native';
import MiniPlayer from '../../Components/MiniPlayer';
import {FlatList} from 'react-native';
import TrackRow from '../../Components/TrackRow';
import {Track} from '../../Types';

const Container = styled.View`
  flex: 1;
`;

export type TracksProps = {
  onGoBack: () => void;
  tracks: Track[];
};

const Tracks: FC<TracksProps> = props => {
  const {tracks, onGoBack} = props;
  return (
    <>
      <Container>
        <Header
          title="Tracks"
          onGoBack={onGoBack}
          onSearch={() => {}}
          onFilter={() => {}}
        />
        <FlatList
          data={tracks}
          renderItem={({item}) => <TrackRow track={item} />}
          keyExtractor={item => item.id}
        />
      </Container>
      <MiniPlayer />
    </>
  );
};

export default Tracks;
