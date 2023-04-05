import React, {FC} from 'react';
import Progressbar from '../Progressbar';
import styled from '@emotion/native';
import Ionicons from 'react-native-vector-icons/Ionicons';
import {PlayerState} from '../../Types';

const Container = styled.View`
  flex: 1;
  width: 93%;
`;

const Time = styled.Text`
  font-family: 'Gilroy-Medium';
  font-size: 12px;
`;

const Duration = styled.Text`
  font-family: 'Gilroy-Medium';
  font-size: 12px;
`;

const TimeContainer = styled.View`
  flex-direction: row;
  justify-content: space-between;
  margin-left: 14px;
  margin-right: 14px;
`;

const Button = styled.TouchableOpacity`
  justify-content: center;
  align-items: center;
`;

const Row = styled.View`
  flex-direction: row;
  justify-content: space-between;
  align-items: center;
  height: 140px;
  margin-left: 14px;
  margin-right: 14px;
`;

export type PlayerControlsProps = {
  onPlay: () => void;
  onPause: () => void;
  onSeek: (value: number) => void;
  onSkipPrevious: () => void;
  onSkipNext: () => void;
  onShuffle: () => void;
  onRepeat: () => void;
  playerState: PlayerState;
};

const PlayerControls: FC<PlayerControlsProps> = props => {
  const {
    onPlay,
    onPause,
    onSeek,
    onSkipPrevious,
    onSkipNext,
    onShuffle,
    onRepeat,
    playerState: {
      isPlaying,
      progress,
      time,
      duration,
      isShuffling,
      isRepeating,
    },
  } = props;
  return (
    <Container>
      <Progressbar
        progress={progress}
        onValueChange={onSeek}
        thumbTintColor="#fff"
      />
      <TimeContainer>
        <Time>{time}</Time>
        <Duration>{duration}</Duration>
      </TimeContainer>
      <Row>
        <Button onPress={onShuffle}>
          {isShuffling && (
            <Ionicons name="ios-shuffle" size={24} color="#fff" />
          )}
          {!isShuffling && (
            <Ionicons name="ios-shuffle" size={24} color="#a7a7a9" />
          )}
        </Button>
        <Button onPress={onSkipPrevious}>
          <Ionicons name="ios-play-skip-back" size={28} color="#fff" />
        </Button>
        {!isPlaying && (
          <Button onPress={onPlay}>
            <Ionicons name="ios-play" size={46} color="#fff" />
          </Button>
        )}
        {isPlaying && (
          <Button onPress={onPause}>
            <Ionicons name="ios-pause" size={46} color="#fff" />
          </Button>
        )}
        <Button onPress={onSkipNext}>
          <Ionicons name="ios-play-skip-forward" size={28} color="#fff" />
        </Button>
        <Button onPress={onRepeat}>
          {isRepeating && <Ionicons name="md-repeat" size={24} color="#fff" />}
          {!isRepeating && (
            <Ionicons name="md-repeat" size={24} color="#a7a7a9" />
          )}
        </Button>
      </Row>
    </Container>
  );
};

export default PlayerControls;
