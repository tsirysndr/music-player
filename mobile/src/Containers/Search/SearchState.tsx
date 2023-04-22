import {atom} from 'recoil';

export const searchState = atom({
  key: 'searchState',
  default: {
    query: '',
    currentFilter: 'Tracks',
    filters: ['Tracks', 'Albums', 'Artists', 'Playlists'],
  },
});
