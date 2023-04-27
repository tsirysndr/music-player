import {atom} from 'recoil';

export const searchState = atom({
  key: 'searchState',
  default: {
    query: '',
    currentFilter: 'Tracks',
    filters: ['Tracks', 'Albums', 'Artists', 'Playlists'],
    results: {
      tracks: [] as any[],
      albums: [] as any[],
      artists: [] as any[],
      playlists: [] as any[],
    },
  },
});
