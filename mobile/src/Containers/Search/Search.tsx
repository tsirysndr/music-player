import styled from '@emotion/native';
import React, {FC} from 'react';
import MiniPlayer from '../../Components/MiniPlayer';
import SearchBar from './SearchBar';
import Filters from './Filters';

const Container = styled.View`
  flex: 1;
  background-color: #000;
`;

const MiniPlayerWrapper = styled.View`
  background-color: #000;
`;

const Search: FC = () => {
  return (
    <>
      <Container>
        <SearchBar />
        <Filters />
      </Container>
      <MiniPlayerWrapper>
        <MiniPlayer />
      </MiniPlayerWrapper>
    </>
  );
};

export default Search;
