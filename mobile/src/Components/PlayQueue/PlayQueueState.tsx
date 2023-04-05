import {atom} from 'recoil';
import {Track} from '../../Types';
import {tracks} from '../../Mocks/Tracks';

export const playQueueState = atom<{
  nextTracks: Track[];
  previousTracks: Track[];
  position: number;
}>({
  key: 'playQueueState',
  default: {
    position: 0,
    nextTracks: tracks.slice(1),
    previousTracks: tracks.slice(0, 1),
  },
});
