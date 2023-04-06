import React, {FC} from 'react';
import MiniPlayer from './MiniPlayer';
import {useRecoilState, useRecoilValue} from 'recoil';
import {currentTrackState} from '../CurrentTrack/CurrentTrackState';
import {playerState} from '../PlayerControls/PlayerState';

const MiniPlayerWithData: FC = () => {
  const currentTrack = useRecoilValue(currentTrackState);
  const [{isPlaying, progress}, setPlayerState] = useRecoilState(playerState);
  const onPlay = () => {
    setPlayerState(prevState => ({...prevState, isPlaying: true}));
  };
  const onPause = () => {
    setPlayerState(prevState => ({...prevState, isPlaying: false}));
  };
  return (
    <MiniPlayer
      track={currentTrack}
      progress={progress}
      onSkipNext={() => {}}
      onPlay={onPlay}
      onPause={onPause}
      playing={isPlaying}
    />
  );
};

export default MiniPlayerWithData;
