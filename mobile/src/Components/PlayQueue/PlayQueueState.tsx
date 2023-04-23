import {atom} from 'recoil';
import {Track} from '../../Types';

export const playQueueState = atom<{
  nextTracks: Track[];
  previousTracks: Track[];
  position: number;
}>({
  key: 'playQueueState',
  default: {
    position: 0,
    nextTracks: [],
    previousTracks: [],
  },
});
