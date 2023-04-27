import React, {FC} from 'react';
import MiniPlayer from './MiniPlayer';
import {useRecoilState, useRecoilValue} from 'recoil';
import {currentTrackState} from '../CurrentTrack/CurrentTrackState';
import {playerState} from '../PlayerControls/PlayerState';
import {useNavigation} from '@react-navigation/native';

const MiniPlayerWithData: FC = () => {
  const navigation = useNavigation<any>();
  const currentTrack = useRecoilValue(currentTrackState);
  const [{isPlaying, progress}, setPlayerState] = useRecoilState(playerState);
  const onPlay = () => {
    setPlayerState(prevState => ({...prevState, isPlaying: true}));
  };
  const onPause = () => {
    setPlayerState(prevState => ({...prevState, isPlaying: false}));
  };
  const onOpenPlayer = () => {
    navigation.navigate('Player');
  };
  return (
    <MiniPlayer
      track={currentTrack}
      progress={progress}
      onSkipNext={() => {}}
      onPlay={onPlay}
      onPause={onPause}
      playing={isPlaying}
      onOpenPlayer={onOpenPlayer}
    />
  );
};

export default MiniPlayerWithData;
