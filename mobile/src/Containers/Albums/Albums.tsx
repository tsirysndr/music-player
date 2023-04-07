import styled from '@emotion/native';
import React, {FC} from 'react';
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
  albums: Album[];
};

const Albums: FC<AlbumsProps> = props => {
  const {albums, onGoBack} = props;
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
          renderItem={({item}) => (
            <AlbumRow album={item} onSelected={() => {}} />
          )}
          keyExtractor={item => item.id}
        />
      </Container>
      <MiniPlayer />
    </>
  );
};

export default Albums;
