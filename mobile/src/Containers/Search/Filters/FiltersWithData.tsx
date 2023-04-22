import React, {FC} from 'react';
import {useRecoilState} from 'recoil';
import Filters from './Filters';
import {searchState} from '../SearchState';

const FiltersWithData: FC = () => {
  const [searchParams, setSearchParams] = useRecoilState(searchState);
  const {query, filters, currentFilter} = searchParams;
  const onSelectFilter = (filter: string) => {
    setSearchParams({
      ...searchParams,
      currentFilter: filter,
    });
  };
  return (
    <Filters
      query={query}
      list={filters}
      current={currentFilter}
      onSelectFilter={onSelectFilter}
    />
  );
};

export default FiltersWithData;
