import React from 'react';
import PlayerControls from './PlayerControls';
import {useRecoilState} from 'recoil';
import {playerState} from './PlayerState';
import {playQueueState} from '../PlayQueue/PlayQueueState';
import {currentTrackState} from '../CurrentTrack/CurrentTrackState';

const PlayerControlsWithData = () => {
  const [currentTrack, setCurrentTrack] = useRecoilState(currentTrackState);
  const [state, setPlayerState] = useRecoilState(playerState);
  const [{previousTracks, nextTracks}, setPlayQueue] =
    useRecoilState(playQueueState);

  const onPlay = () => {
    setPlayerState(prevState => ({...prevState, isPlaying: true}));
  };

  const onPause = () => {
    setPlayerState(prevState => ({...prevState, isPlaying: false}));
  };

  const onSkipNext = () => {
    if (nextTracks.length === 0) {
      return;
    }
  };

  const onShuffle = () => {
    setPlayerState(prevState => ({
      ...prevState,
      isShuffling: !prevState.isShuffling,
    }));
  };

  const onRepeat = () => {
    setPlayerState(prevState => ({
      ...prevState,
      isRepeating: !prevState.isRepeating,
    }));
  };

  const onSkipPrevious = () => {
    if (previousTracks.length === 0) {
      return;
    }
  };

  return (
    <PlayerControls
      playerState={state}
      onPlay={onPlay}
      onPause={onPause}
      onSkipNext={onSkipNext}
      onSkipPrevious={onSkipPrevious}
      onSeek={() => {}}
      onShuffle={onShuffle}
      onRepeat={onRepeat}
    />
  );
};

export default PlayerControlsWithData;
