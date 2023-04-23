import styled from '@emotion/native';
import React, {FC, useState, useEffect} from 'react';
import Feather from 'react-native-vector-icons/Feather';
import Ionicons from 'react-native-vector-icons/Ionicons';
import {Keyboard} from 'react-native';
import {useCover} from '../../Hooks/useCover';

const Container = styled.TouchableOpacity`
  flex-direction: row;
  height: 60px;
  width: 100%;
  background-color: #000;
`;

const TrackTitle = styled.Text`
  color: #fff;
  font-family: 'Gilroy-Bold';
  font-size: 16px;
`;

const TrackArtist = styled.Text`
  color: #a7a7a9;
  font-family: 'Gilroy-Bold';
  font-size: 14px;
  margin-top: 2px;
`;

const TrackInfo = styled.View`
  flex-direction: column;
  margin-left: 20px;
  justify-content: center;
  flex: 1;
`;

const Cover = styled.Image`
  width: 60px;
  height: 60px;
  border-radius: 2px;
`;

const NoCover = styled.View`
  width: 60px;
  height: 60px;
  background-color: #161515;
  align-items: center;
  justify-content: center;
  border-radius: 3px;
`;

const ProgressbarContainer = styled.View`
  width: 100%;
  height: 1.5px;
  background-color: #4f4f4f;
`;

const ProgressbarFill = styled.View<{progress: number}>`
  width: 0%;
  height: 100%;
  background-color: #ab28fc;
  ${({progress}) => `width: ${progress}%`};
`;

const Button = styled.TouchableOpacity`
  width: 60px;
  height: 60px;
  align-items: center;
  justify-content: center;
`;

export type ProgressbarProps = {
  progress: number;
};
const Progressbar: FC<ProgressbarProps> = props => {
  return (
    <ProgressbarContainer>
      <ProgressbarFill {...props} />
    </ProgressbarContainer>
  );
};

export type MiniPlayerProps = {
  track: any;
  playing: boolean;
  progress: number;
  onPlay: () => void;
  onPause: () => void;
  onSkipNext: () => void;
  onOpenPlayer: () => void;
};

const MiniPlayer: FC<MiniPlayerProps> = props => {
  const {track, playing, progress, onPlay, onPause, onSkipNext, onOpenPlayer} =
    props;
  const [keyboardVisible, setKeyboardVisible] = useState(false);
  const cover = useCover(track.cover);

  // listen to keyboard events
  useEffect(() => {
    const subscription = Keyboard.addListener('keyboardDidShow', () => {
      setKeyboardVisible(true);
    });
    const subscription2 = Keyboard.addListener('keyboardDidHide', () => {
      setKeyboardVisible(false);
    });
    return () => {
      subscription.remove();
      subscription2.remove();
    };
  }, []);

  if (keyboardVisible) {
    return null;
  }

  return (
    <>
      <Progressbar progress={progress} />
      <Container onPress={onOpenPlayer}>
        <>
          {!track.cover && (
            <NoCover>
              <Feather name="music" size={30} color="#a7a7a9" />
            </NoCover>
          )}
          {track.cover && <Cover source={{uri: cover}} />}
        </>
        <TrackInfo>
          <TrackTitle numberOfLines={1}>{track.title}</TrackTitle>
          <TrackArtist numberOfLines={1}>{track.artist}</TrackArtist>
        </TrackInfo>
        <>
          {!playing && (
            <Button onPress={onPlay}>
              <Ionicons name="play" size={24} color="#fff" />
            </Button>
          )}
          {playing && (
            <Button onPress={onPause}>
              <Ionicons name="pause" size={24} color="#fff" />
            </Button>
          )}
        </>
        <Button onPress={onSkipNext}>
          <Ionicons name="play-skip-forward" size={24} color="#fff" />
        </Button>
      </Container>
    </>
  );
};

export default MiniPlayer;
