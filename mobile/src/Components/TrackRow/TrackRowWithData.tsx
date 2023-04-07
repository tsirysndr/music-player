import React, {FC} from 'react';
import TrackRow from './TrackRow';
import {useRecoilState} from 'recoil';
import {currentTrackState} from '../CurrentTrack/CurrentTrackState';
import {Track} from '../../Types';

export type TrackRowWithDataProps = {
  track: Track;
};

const TrackRowWithData: FC<TrackRowWithDataProps> = ({track}) => {
  const [currentTrack, setCurrentTrack] = useRecoilState(currentTrackState);
  const onPlay = (item: Track) => {
    setCurrentTrack(item);
  };
  return <TrackRow track={track} currentTrack={currentTrack} onPlay={onPlay} />;
};

export default TrackRowWithData;
