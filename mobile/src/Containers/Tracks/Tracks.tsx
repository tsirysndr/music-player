import React, {FC, useCallback} from 'react';
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
  fetchMore: () => void;
  onSearch: () => void;
  onFilter: () => void;
};

const Tracks: FC<TracksProps> = props => {
  const {tracks, onGoBack, fetchMore, onSearch, onFilter} = props;
  const renderItem = useCallback(
    ({item}: {item: Track}) => <TrackRow track={item} />,
    [],
  );
  return (
    <>
      <Container>
        <Header
          title="Tracks"
          onGoBack={onGoBack}
          onSearch={onSearch}
          onFilter={onFilter}
        />
        <FlatList
          data={tracks}
          renderItem={renderItem}
          keyExtractor={item => item.id}
          onEndReached={fetchMore}
          onEndReachedThreshold={0.5}
        />
      </Container>
      <MiniPlayer />
    </>
  );
};

export default Tracks;
