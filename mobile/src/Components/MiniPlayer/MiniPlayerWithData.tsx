import React, {FC} from 'react';
import MiniPlayer from './MiniPlayer';
import {useRecoilValue} from 'recoil';
import {currentTrackState} from '../CurrentTrack/CurrentTrackState';

const MiniPlayerWithData: FC = () => {
  const currentTrack = useRecoilValue(currentTrackState);
  return (
    <MiniPlayer
      track={currentTrack}
      progress={30}
      onSkipNext={() => {}}
      onPlay={() => {}}
      onPause={() => {}}
      playing={true}
    />
  );
};

export default MiniPlayerWithData;
