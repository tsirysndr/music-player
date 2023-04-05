import styled from '@emotion/native';
import React, {FC} from 'react';
import MiniPlayer from '../../Components/MiniPlayer';
import Header from '../../Components/Header';

const Container = styled.View`
  flex: 1;
`;

export type AlbumsProps = {
  onGoBack: () => void;
};

const Albums: FC<AlbumsProps> = props => {
  const {onGoBack} = props;
  return (
    <>
      <Container>
        <Header
          title="Albums"
          onGoBack={onGoBack}
          onSearch={() => {}}
          onFilter={() => {}}
        />
      </Container>
      <MiniPlayer />
    </>
  );
};

export default Albums;