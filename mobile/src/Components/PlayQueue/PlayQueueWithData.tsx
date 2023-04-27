import React, {FC} from 'react';
import PlayQueue from './PlayQueue';
import {playQueueState} from './PlayQueueState';
import {useRecoilValue} from 'recoil';
import {currentTrackState} from '../CurrentTrack/CurrentTrackState';

const PlayQueueWithData: FC = () => {
  const currentTrack = useRecoilValue(currentTrackState);
  const {nextTracks} = useRecoilValue(playQueueState);
  return (
    <PlayQueue
      currentTrack={currentTrack}
      nextTracks={nextTracks}
      onPlayItem={(_item: any) => {}}
    />
  );
};

export default PlayQueueWithData;
