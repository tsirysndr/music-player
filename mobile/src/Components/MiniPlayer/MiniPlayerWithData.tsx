import React, {FC} from 'react';
import MiniPlayer from './MiniPlayer';
import {tracks} from '../../Mocks/Tracks';

const MiniPlayerWithData: FC = () => {
  return (
    <MiniPlayer
      track={tracks[0]}
      progress={30}
      onSkipNext={() => {}}
      onPlay={() => {}}
      onPause={() => {}}
      playing={true}
    />
  );
};

export default MiniPlayerWithData;
