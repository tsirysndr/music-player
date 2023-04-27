import React, {FC} from 'react';
import Player from './Player';
import {useRecoilValue} from 'recoil';
import {currentTrackState} from '../../Components/CurrentTrack/CurrentTrackState';

const PlayerWithData: FC = () => {
  const currentTrack = useRecoilValue(currentTrackState);
  return <Player track={currentTrack} />;
};

export default PlayerWithData;
