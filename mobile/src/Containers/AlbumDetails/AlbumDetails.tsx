import styled from '@emotion/native';
import React, {FC} from 'react';
import MiniPlayer from '../../Components/MiniPlayer';
import {useCover} from '../../Hooks/useCover';
import Feather from 'react-native-vector-icons/Feather';
import DefaultLinearGradient from 'react-native-linear-gradient';
import Shuffle from '../../Components/Icons/Shuffle';
import Play from '../../Components/Icons/Play';
import TrackRow from '../../Components/TrackRow';

const Container = styled.View`
  flex: 1;
  background-color: #000;
`;

const ScrollContainer = styled.ScrollView`
  flex: 1;
  background-color: #000;
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

const AlbumCover = styled.Image`
  width: 200px;
  height: 200px;
  border-radius: 6px;
`;

const NoAlbumCover = styled.View`
  width: 200px;
  height: 200px;
  background-color: #161515;
  border-radius: 6px;
  align-items: center;
  justify-content: center;
`;

const AlbumCoverBackground = styled.ImageBackground`
  width: 100%;
  height: 400px;
  justify-content: center;
  align-items: center;
`;

const NoAlbumCoverBackground = styled.View`
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

const DarkBackground = styled.View`
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  height: 400px;
  background-color: rgba(0, 0, 0, 0.5);
`;

const AlbumTitle = styled.Text`
  font-family: Gilroy-Bold;
  color: #fff;
  margin-top: 20px;
  font-size: 18px;
  text-align: center;
`;

const Artist = styled.Text`
  font-family: Gilroy-Bold;
  color: #b3b2b2;
  margin-top: 5px;
  font-size: 14px;
  text-align: center;
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
`;

const AlbumInfo = styled.View`
  margin-left: 25px;
  margin-right: 25px;
`;

export type AlbumDetailsProps = {
  album: any;
  onGoBack: () => void;
};

const AlbumDetails: FC<AlbumDetailsProps> = props => {
  const {album, onGoBack} = props;
  const cover = useCover(album.cover);
  return (
    <Container>
      <BackButton onPress={onGoBack}>
        <Feather name="arrow-left" size={24} color="#fff" />
      </BackButton>
      <ScrollContainer>
        <Container>
          <>
            {album.cover && (
              <AlbumCoverBackground source={{uri: cover}}>
                <DarkBackground />
                <LinearGradient colors={['transparent', 'rgba(0, 0, 0, 1)']} />
                <AlbumCover source={{uri: cover}} />
                <AlbumInfo>
                  <AlbumTitle>{album.title}</AlbumTitle>
                  <Artist>Album by {album.artist}</Artist>
                </AlbumInfo>
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
              </AlbumCoverBackground>
            )}
            {!album.cover && (
              <NoAlbumCoverBackground>
                <NoAlbumCover>
                  <Feather name="disc" size={100} color="#bdb9b958" />
                </NoAlbumCover>
                <AlbumInfo>
                  <AlbumTitle>{album.title}</AlbumTitle>
                  <Artist>Album by {album.artist}</Artist>
                </AlbumInfo>
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
              </NoAlbumCoverBackground>
            )}
          </>
          {album.tracks.map((track: any, index: number) => (
            <TrackRow key={index} track={track} showAlbum={false} />
          ))}
        </Container>
      </ScrollContainer>
      <MiniPlayer />
    </Container>
  );
};

export default AlbumDetails;
