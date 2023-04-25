import styled from '@emotion/native';
import React, {FC} from 'react';
import {FlatList} from 'react-native';
import MiniPlayer from '../../Components/MiniPlayer';
import Feather from 'react-native-vector-icons/Feather';
import DefaultLinearGradient from 'react-native-linear-gradient';
import {Artist} from '../../Types';
import TrackRow from '../../Components/TrackRow';
import {Album} from '../../Components/Albums/Albums';
import SectionHeader from '../../Components/SectionHeader';
import Shuffle from '../../Components/Icons/Shuffle';
import Play from '../../Components/Icons/Play';
import SvgMic from '../../Components/Icons/Mic';

const ScrollContainer = styled.ScrollView`
  flex: 1;
  background-color: #000;
`;

const Container = styled.View`
  flex: 1;
`;

const ArtistCoverBackground = styled.ImageBackground`
  width: 100%;
  height: 400px;
  justify-content: center;
  align-items: center;
`;

const NoArtistCoverBackground = styled.View`
  width: 100%;
  height: 350px;
  justify-content: center;
  align-items: center;
`;

const LinearGradient = styled(DefaultLinearGradient)`
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  height: 400px;
`;

const TracksWrapper = styled.View`
  margin-top: 50px;
`;

const AlbumsWrapper = styled.View`
  margin-top: 50px;
  margin-bottom: 100px;
`;

const BackButton = styled.TouchableOpacity`
  width: 50px;
  height: 50px;
  justify-content: center;
  align-items: center;
  position: absolute;
  top: 0;
  left: 0;
  z-index: 1;
`;

const PlayButton = styled.TouchableOpacity`
  flex-direction: row;
  width: 140px;
  height: 40px;
  border-radius: 20px;
  margin-right: 20px;
  background-color: #ab28fc;
  justify-content: center;
  align-items: center;
`;

const PlayButtonText = styled.Text`
  font-family: Gilroy-Bold;
  color: #fff;
  font-size: 14px;
  margin-left: 10px;
`;

const ShuffleButton = styled.TouchableOpacity`
  flex-direction: row;
  width: 140px;
  height: 40px;
  border-radius: 20px;
  justify-content: center;
  align-items: center;
  background-color: #ab28fc28;
`;

const ShuffleButtonText = styled.Text`
  font-family: Gilroy-Bold;
  color: #ab28fc;
  font-size: 14px;
  margin-left: 10px;
`;

const Buttons = styled.View`
  flex-direction: row;
  justify-content: space-between;
  margin-top: 25px;
  margin-bottom: 5px;
`;

const ArtistInfo = styled.View`
  flex: 1;
  justify-content: flex-end;
`;

const ArtistName = styled.Text`
  font-family: Gilroy-Bold;
  color: #fff;
  font-size: 24px;
`;

const ArtistWrapper = styled.View`
  flex: 1;
  align-items: center;
  justify-content: center;
`;

const NoArtistCover = styled.View`
  width: 150px;
  height: 150px;
  border-radius: 80px;
  justify-content: center;
  align-items: center;
  background-color: #161515;
`;

export type ArtistDetailsProps = {
  artist: Artist;
  onGoBack: () => void;
  onPressAlbum: (album: any) => void;
  onSeeAllAlbums: () => void;
  onSeeAllTracks: () => void;
};

const ArtistDetails: FC<ArtistDetailsProps> = props => {
  const {artist, onGoBack, onPressAlbum, onSeeAllTracks, onSeeAllAlbums} =
    props;
  return (
    <Container>
      <BackButton onPress={onGoBack}>
        <Feather name="arrow-left" size={24} color="#fff" />
      </BackButton>
      <ScrollContainer>
        <Container>
          {artist.picture && (
            <ArtistCoverBackground source={{uri: artist.picture}}>
              <LinearGradient colors={['transparent', 'rgba(0, 0, 0, 1)']} />
              <ArtistInfo>
                <ArtistName>{artist.name}</ArtistName>
              </ArtistInfo>
              <Buttons>
                <PlayButton>
                  <Play color="#fff" small />
                  <PlayButtonText>Play</PlayButtonText>
                </PlayButton>
                <ShuffleButton>
                  <Shuffle color="#ab28fc" />
                  <ShuffleButtonText>Shuffle</ShuffleButtonText>
                </ShuffleButton>
              </Buttons>
            </ArtistCoverBackground>
          )}
          {!artist.picture && (
            <NoArtistCoverBackground>
              <ArtistWrapper>
                <NoArtistCover>
                  <SvgMic height={70} width={70} />
                </NoArtistCover>
              </ArtistWrapper>
              <ArtistName>{artist.name}</ArtistName>
              <Buttons>
                <PlayButton>
                  <Play color="#fff" small />
                  <PlayButtonText>Play</PlayButtonText>
                </PlayButton>
                <ShuffleButton>
                  <Shuffle color="#ab28fc" />
                  <ShuffleButtonText>Shuffle</ShuffleButtonText>
                </ShuffleButton>
              </Buttons>
            </NoArtistCoverBackground>
          )}
          <TracksWrapper>
            <SectionHeader
              title="Tracks"
              onSeeAll={onSeeAllTracks}
              showSeeAll={!!artist.tracks?.length}
            />
            {artist.tracks?.slice(0, 5).map((track: any, index: number) => (
              <TrackRow key={index} track={track} />
            ))}
          </TracksWrapper>
          <AlbumsWrapper>
            <SectionHeader
              title="Albums"
              onSeeAll={onSeeAllAlbums}
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
