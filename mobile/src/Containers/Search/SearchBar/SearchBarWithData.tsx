import React, {FC} from 'react';
import SearchBar from './SearchBar';

const SearchBarWithData: FC = () => {
  const onChange = (_value: string) => {};
  return <SearchBar onChange={onChange} />;
};

export default SearchBarWithData;
