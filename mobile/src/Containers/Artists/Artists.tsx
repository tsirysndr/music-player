import styled from '@emotion/native';
import React, {FC} from 'react';
import MiniPlayer from '../../Components/MiniPlayer';
import Header from '../../Components/Header';

const Container = styled.View`
  flex: 1;
`;

export type ArtistsProps = {
  onGoBack: () => void;
};

const Artists: FC<ArtistsProps> = props => {
  const {onGoBack} = props;
  return (
    <>
      <Container>
        <Header
          title="Artists"
          onGoBack={onGoBack}
          onSearch={() => {}}
          onFilter={() => {}}
        />
      </Container>
      <MiniPlayer />
    </>
  );
};

export default Artists;
