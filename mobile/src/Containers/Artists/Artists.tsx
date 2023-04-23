import styled from '@emotion/native';
import React, {FC} from 'react';
import MiniPlayer from '../../Components/MiniPlayer';
import Header from '../../Components/Header';
import {Artist} from '../../Types';
import ArtistRow from '../../Components/ArtistRow';
import {FlatList} from 'react-native';

const Container = styled.View`
  flex: 1;
`;

export type ArtistsProps = {
  onGoBack: () => void;
  artists: Artist[];
  fetchMore: () => void;
};

const Artists: FC<ArtistsProps> = props => {
  const {artists, onGoBack, fetchMore} = props;
  return (
    <>
      <Container>
        <Header
          title="Artists"
          onGoBack={onGoBack}
          onSearch={() => {}}
          onFilter={() => {}}
        />
        <FlatList
          data={artists}
          renderItem={({item}) => (
            <ArtistRow artist={item} onSelected={() => {}} />
          )}
          keyExtractor={item => item.id}
          onEndReached={fetchMore}
          onEndReachedThreshold={0.5}
        />
      </Container>
      <MiniPlayer />
    </>
  );
};

export default Artists;
