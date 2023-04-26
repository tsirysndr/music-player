import React, {FC} from 'react';
import styled from '@emotion/native';
import Header from '../../Components/Header';

const Container = styled.View`
  flex: 1;
  background-color: #000;
`;

export type FilterProps = {
  onGoBack: () => void;
  placeholder: string;
  onSearch: (query: string) => void;
  albums?: any[];
  artists?: any[];
  tracks?: any[];
};

const Filter: FC<FilterProps> = props => {
  const {placeholder, onGoBack, onSearch, albums, artists, tracks} = props;
  return (
    <>
      <Container>
        <Header
          placeholder={placeholder}
          onGoBack={onGoBack}
          onKeyPress={onSearch}
        />
        {albums && <></>}
        {tracks && <></>}
        {artists && <></>}
      </Container>
    </>
  );
};

export default Filter;
