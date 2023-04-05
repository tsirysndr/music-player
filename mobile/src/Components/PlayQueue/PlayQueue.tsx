import React, {FC} from 'react';
import {Track} from '../../Types';

export type PlayQueueProps = {
  currentTrack?: Track;
  nextTracks: Track[];
  onPlayItem: (item: any) => void;
};

const PlayQueue: FC<PlayQueueProps> = () => {
  return <></>;
};

export default PlayQueue;
