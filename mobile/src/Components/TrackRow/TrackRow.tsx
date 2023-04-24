import React, {FC} from 'react';
import {Track} from '../../Types';
import styled, {css} from '@emotion/native';
import Feather from 'react-native-vector-icons/Feather';
import Ionicons from 'react-native-vector-icons/Ionicons';
import {useCover} from '../../Hooks/useCover';

const Container = styled.View`
  height: 80px;
  width: 100%;
  flex-direction: row;
  align-items: center;
  padding-left: 20px;
`;

const Cover = styled.Image`
  width: 60px;
  height: 60px;
`;

const NoAlbumCover = styled.View`
  width: 60px;
  height: 60px;
  background-color: #161515;
  align-items: center;
  justify-content: center;
`;

const Title = styled.Text<{active: boolean}>`
  color: #fff;
  font-family: 'Gilroy-Bold';
  font-size: 16px;
  ${props =>
    props.active &&
    css`
      color: #ab28fc;
    `}
`;

const Artist = styled.Text`
  color: #a7a7a9;
  font-family: 'Gilroy-Bold';
  font-size: 14px;
  margin-top: 2px;
`;

const AlbumInfo = styled.View`
  flex-direction: column;
  margin-left: 15px;
  flex: 1;
`;

const Button = styled.TouchableOpacity`
  height: 40px;
  width: 40px;
  align-items: center;
  justify-content: center;
  z-index: 1;
`;

const TrackWrapper = styled.View`
  background-color: #232323;
`;

const TouchableTrack = styled.TouchableOpacity`
  background-color: #000;
`;

const TrackNumber = styled.Text`
  color: #fff;
  font-family: 'Gilroy-Bold';
  font-size: 16px;
`;

const TrackNumberWrapper = styled.View`
  width: 30px;
  height: 60px;
  align-items: center;
  justify-content: center;
`;

export type TrackRowProps = {
  track: Track;
  currentTrack?: Track;
  onPlay: (item: Track) => void;
  showAlbum?: boolean;
};

const TrackRow: FC<TrackRowProps> = props => {
  const {track, currentTrack, onPlay, showAlbum} = props;
  const cover = useCover(track.cover);
  return (
    <TrackWrapper>
      <TouchableTrack onPress={() => onPlay(track)}>
        <Container>
          {showAlbum && (
            <>
              {track.cover && <Cover source={{uri: cover}} />}
              {!track.cover && (
                <NoAlbumCover>
                  <Feather name="disc" size={40} color="#a7a7a9" />
                </NoAlbumCover>
              )}
            </>
          )}
          {!showAlbum && (
            <TrackNumberWrapper>
              <TrackNumber>{track.trackNumber}</TrackNumber>
            </TrackNumberWrapper>
          )}
          <AlbumInfo>
            <Title
              numberOfLines={1}
              ellipsizeMode="tail"
              active={track.id === currentTrack?.id}>
              {track.title}
            </Title>
            <Artist numberOfLines={1} ellipsizeMode="tail">
              {track.artist}
            </Artist>
          </AlbumInfo>
          <Button>
            <Ionicons name="ellipsis-vertical" color={'#ffffff99'} size={18} />
          </Button>
        </Container>
      </TouchableTrack>
    </TrackWrapper>
  );
};

TrackRow.defaultProps = {
  showAlbum: true,
};

export default TrackRow;
