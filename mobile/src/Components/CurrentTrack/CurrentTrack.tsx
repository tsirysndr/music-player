import React, {FC} from 'react';
import styled from '@emotion/native';
import Feather from 'react-native-vector-icons/Feather';
import FontAwesome from 'react-native-vector-icons/FontAwesome5';
import Ionicons from 'react-native-vector-icons/Ionicons';
import {Dimensions} from 'react-native';

const NoAlbumCover = styled.View`
  width: 180px;
  height: 180px;
  background-color: #161515;
  margin-right: 8px;
  margin-left: 8px;
  border-radius: 3px;
  align-items: center;
  justify-content: center;
`;

const Cover = styled.Image<{width: number}>`
  margin-right: 8px;
  margin-left: 8px;
  border-radius: 3px;
  ${({width}) => `width: ${width}px`};
  ${({width}) => `height: ${width}px`};
`;

const Container = styled.View``;

const TrackRow = styled.View`
  flex-direction: row;
  align-items: center;
  justify-content: space-between;
  height: 120px;
  margin-left: 10px;
  margin-right: 10px;
`;

const TrackInfo = styled.View`
  flex-direction: column;
  flex: 1;
`;

const TrackTitle = styled.Text`
  color: #fff;
  font-family: 'Gilroy-Bold';
  font-size: 20px;
`;

const TrackArtist = styled.Text`
  color: #a7a7a9;
  font-family: 'Gilroy-Bold';
  font-size: 14px;
  margin-top: 3px;
`;

const Button = styled.TouchableOpacity`
  height: 40px;
  width: 40px;
  justify-content: center;
  align-items: flex-end;
`;

export type CurrentTrackProps = {
  track: any;
};

const CurrentTrack: FC<CurrentTrackProps> = ({track}) => {
  const screenWidth = Math.round(Dimensions.get('window').width);
  return (
    <Container>
      <>
        {track?.cover && (
          <Cover source={{uri: track.cover}} width={screenWidth - 60} />
        )}
        {!track?.cover && (
          <NoAlbumCover>
            <Feather name="music" size={50} color="#a7a7a9" />
          </NoAlbumCover>
        )}
        <TrackRow>
          <TrackInfo>
            <TrackTitle>{track.title}</TrackTitle>
            <TrackArtist>{track.artist}</TrackArtist>
          </TrackInfo>
          <Button>
            <Ionicons name="heart-outline" size={24} color="#cbcbcb" />
          </Button>
          <Button>
            <FontAwesome name="ellipsis-v" size={20} color="#cbcbcb" />
          </Button>
        </TrackRow>
      </>
    </Container>
  );
};

export default CurrentTrack;
