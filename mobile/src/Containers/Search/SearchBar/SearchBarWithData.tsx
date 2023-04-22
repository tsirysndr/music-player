import React, {FC} from 'react';
import SearchBar from './SearchBar';
import {useRecoilState} from 'recoil';
import {searchState} from '../SearchState';

const SearchBarWithData: FC = () => {
  const [searchParams, setSearchParams] = useRecoilState(searchState);
  const {query} = searchParams;
  const onChange = (value: string) => {
    const currentFilter = value === '' ? 'Tracks' : searchParams.currentFilter;
    setSearchParams({
      ...searchParams,
      query: value,
      currentFilter,
    });
  };
  return <SearchBar query={query} onChange={onChange} />;
};

export default SearchBarWithData;
