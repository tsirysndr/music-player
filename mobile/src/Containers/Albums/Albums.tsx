import styled from '@emotion/native';
import React, {FC, useCallback} from 'react';
import MiniPlayer from '../../Components/MiniPlayer';
import Header from '../../Components/Header';
import {Album} from '../../Types';
import {FlatList} from 'react-native';
import AlbumRow from '../../Components/AlbumRow';

const Container = styled.View`
  flex: 1;
`;

export type AlbumsProps = {
  onGoBack: () => void;
  fetchMore: () => void;
  albums: Album[];
};

const Albums: FC<AlbumsProps> = props => {
  const {albums, onGoBack, fetchMore} = props;

  const renderItem = useCallback(
    ({item}: {item: Album}) => <AlbumRow album={item} onSelected={() => {}} />,
    [],
  );

  return (
    <>
      <Container>
        <Header
          title="Albums"
          onGoBack={onGoBack}
          onSearch={() => {}}
          onFilter={() => {}}
        />
        <FlatList
          data={albums}
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

export default Albums;
