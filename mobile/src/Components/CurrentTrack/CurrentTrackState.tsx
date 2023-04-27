import {atom} from 'recoil';
import {Track} from '../../Types';

export const currentTrackState = atom<Track | undefined>({
  key: 'currentTrackState',
  default: undefined,
});
