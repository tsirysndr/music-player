import {atom} from 'recoil';
import {Album, Track} from '../../Types';

export const contextMenuState = atom<{
  visible: boolean;
  type: 'album' | 'track' | 'artist' | '';
  item?: Album | Track;
  enablePlayNext?: boolean;
}>({
  key: 'contextMenuState',
  default: {
    visible: false,
    type: '',
    item: undefined,
    enablePlayNext: true,
  },
});
