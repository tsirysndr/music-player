import React, {FC, useCallback} from 'react';
import styled from '@emotion/native';
import {FlatList} from 'react-native';
import Header from '../../Components/Header';
import ArtistRow from '../../Components/ArtistRow';
import TrackRow from '../../Components/TrackRow';
import {Album, Track} from '../../Types';
import AlbumRow from '../../Components/AlbumRow';
import MiniPlayer from '../../Components/MiniPlayer';

const Container = styled.View`
  flex: 1;
  background-color: #000;
`;

export type FilterProps = {
  onGoBack: () => void;
  placeholder: string;
  onSearch: (query: string) => void;
  albums?: any[];
  artists?: any[];
  tracks?: any[];
  onPressArtist: (artist: any) => void;
  onPressAlbum: (album: any) => void;
};

const Filter: FC<FilterProps> = props => {
  const {
    placeholder,
    onGoBack,
    onSearch,
    albums,
    artists,
    tracks,
    onPressArtist,
    onPressAlbum,
  } = props;
  const renderAlbum = useCallback(
    ({item}: {item: Album}) => (
      <AlbumRow album={item} onPressAlbum={() => onPressAlbum(item)} />
    ),
    [onPressAlbum],
  );
  const renderTrack = ({item}: {item: Track}) => <TrackRow track={item} />;
  const renderArtist = useCallback(
    ({item}: {item: any}) => (
      <ArtistRow artist={item} onSelected={() => onPressArtist(item)} />
    ),
    [onPressArtist],
  );
  return (
    <>
      <Container>
        <Header
          placeholder={placeholder}
          onGoBack={onGoBack}
          onKeyPress={onSearch}
        />
        {albums?.length! > 0 && (
          <FlatList
            data={albums}
            renderItem={renderAlbum}
            keyExtractor={item => item.id}
          />
        )}
        {tracks?.length! > 0 && (
          <FlatList
            data={tracks}
            renderItem={renderTrack}
            keyExtractor={item => item.id}
          />
        )}
        {artists && (
          <FlatList
            data={artists}
            renderItem={renderArtist}
            keyExtractor={item => item.id}
          />
        )}
      </Container>
      <MiniPlayer />
    </>
  );
};

export default Filter;
