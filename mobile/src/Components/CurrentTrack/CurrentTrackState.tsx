import {atom} from 'recoil';
import {Track} from '../../Types';
import {tracks} from '../../Mocks/Tracks';

export const currentTrackState = atom<Track | undefined>({
  key: 'currentTrackState',
  default: tracks[0],
});
