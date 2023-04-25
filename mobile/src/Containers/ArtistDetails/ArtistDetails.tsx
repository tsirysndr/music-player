import styled from '@emotion/native';
import React, {FC} from 'react';
import {FlatList} from 'react-native';
import MiniPlayer from '../../Components/MiniPlayer';
import Header from '../../Components/Header';
import {Artist} from '../../Types';
import TrackRow from '../../Components/TrackRow';
import {Album} from '../../Components/Albums/Albums';
import SectionHeader from '../../Components/SectionHeader';

const ScrollContainer = styled.ScrollView`
  flex: 1;
  background-color: #000;
`;

const Container = styled.View`
  flex: 1;
`;

const TracksWrapper = styled.View`
  margin-top: 50px;
`;

const AlbumsWrapper = styled.View`
  margin-top: 50px;
  margin-bottom: 100px;
`;

export type ArtistDetailsProps = {
  artist: Artist;
  onGoBack: () => void;
  onPressAlbum: (album: any) => void;
};

const ArtistDetails: FC<ArtistDetailsProps> = props => {
  const {artist, onGoBack, onPressAlbum} = props;
  return (
    <Container>
      <ScrollContainer>
        <Container>
          <TracksWrapper>
            <SectionHeader
              title="Tracks"
              onSeeAll={() => {}}
              showSeeAll={!!artist.tracks?.length}
            />
            {artist.tracks?.slice(0, 5).map((track: any, index: number) => (
              <TrackRow key={index} track={track} />
            ))}
          </TracksWrapper>
          <AlbumsWrapper>
            <SectionHeader
              title="Albums"
              onSeeAll={() => {}}
              showSeeAll={!!artist.albums?.length}
            />
            <FlatList
              data={artist.albums!}
              showsHorizontalScrollIndicator={false}
              renderItem={({item}) => (
                <Album key={item.id} album={item} onPress={onPressAlbum} />
              )}
              horizontal={true}
            />
          </AlbumsWrapper>
        </Container>
      </ScrollContainer>
      <MiniPlayer />
    </Container>
  );
};

export default ArtistDetails;
