import styled from '@emotion/native';
import React, {FC} from 'react';
import MiniPlayer from '../../Components/MiniPlayer';

const Container = styled.View`
  flex: 1;
`;

const Title = styled.Text`
  color: #fff;
  font-family: 'Gilroy-Bold';
  font-size: 24px;
`;

const Search: FC = () => {
  return (
    <>
      <Container>
        <Title>Search</Title>
      </Container>
      <MiniPlayer />
    </>
  );
};

export default Search;