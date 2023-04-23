import React, {FC} from 'react';
import SearchBar from './SearchBar';
import {useRecoilState} from 'recoil';
import {searchState} from '../SearchState';
import {useSearchLazyQuery} from '../../../Hooks/GraphQL';

const SearchBarWithData: FC = () => {
  const [search] = useSearchLazyQuery();
  const [searchParams, setSearchParams] = useRecoilState(searchState);
  const {query} = searchParams;

  const onChange = (value: string) => {
    const currentFilter = value === '' ? 'Tracks' : searchParams.currentFilter;
    const state = {
      ...searchParams,
      query: value,
      currentFilter,
    };
    setSearchParams(state);

    if (value === '') {
      return;
    }

    search({
      variables: {
        keyword: value,
      },
    }).then(({data}) => {
      setSearchParams({
        ...state,
        results: {
          tracks: data?.search?.tracks || [],
          albums: data?.search?.albums || [],
          artists: data?.search?.artists || [],
          playlists: [],
        },
      });
    });
  };

  return <SearchBar query={query} onChange={onChange} />;
};

export default SearchBarWithData;
