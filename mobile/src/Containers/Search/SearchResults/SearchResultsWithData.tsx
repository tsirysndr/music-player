import React, {FC} from 'react';
import SearchResults from './SearchResults';
import {useRecoilValue} from 'recoil';
import {searchState} from '../SearchState';

const SearchWithData: FC = () => {
  const props = useRecoilValue(searchState);
  return <SearchResults {...props} />;
};

export default SearchWithData;
