import {atom} from 'recoil';
import {PlayerState} from '../../Types';

export const playerState = atom<PlayerState>({
  key: 'playerState',
  default: {
    isPlaying: false,
    isShuffling: false,
    isRepeating: false,
    progress: 30,
    time: '01:13',
    duration: '03:45',
  },
});
