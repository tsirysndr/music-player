import React, {FC} from 'react';
import TrackRow from './TrackRow';
import {useRecoilState} from 'recoil';
import {currentTrackState} from '../CurrentTrack/CurrentTrackState';
import {Track} from '../../Types';
import {playQueueState} from '../PlayQueue/PlayQueueState';

export type TrackRowWithDataProps = {
  track: Track;
};

const TrackRowWithData: FC<TrackRowWithDataProps> = ({track}) => {
  const [currentTrack, setCurrentTrack] = useRecoilState(currentTrackState);
  const [playQueue, setPlayQueue] = useRecoilState(playQueueState);
  const onPlay = (item: Track) => {
    setCurrentTrack(item);
    setPlayQueue({
      ...playQueue,
      previousTracks: playQueue.previousTracks.concat(item),
      position: playQueue.previousTracks.length,
    });
  };
  return <TrackRow track={track} currentTrack={currentTrack} onPlay={onPlay} />;
};

export default TrackRowWithData;
