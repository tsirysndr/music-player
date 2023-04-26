import {atom} from 'recoil';
import {Album, Track} from '../../Types';

export const contextMenuState = atom<{
  visible: boolean;
  type: string;
  item?: Album | Track;
}>({
  key: 'contextMenuState',
  default: {
    visible: false,
    type: '',
    item: undefined,
  },
});
