import React, {FC} from 'react';
import Header from '../../Components/Header';
import styled from '@emotion/native';
import MiniPlayer from '../../Components/MiniPlayer';

const Container = styled.View`
  flex: 1;
`;

export type TracksProps = {
  onGoBack: () => void;
};

const Tracks: FC<TracksProps> = props => {
  const {onGoBack} = props;
  return (
    <>
      <Container>
        <Header
          title="Tracks"
          onGoBack={onGoBack}
          onSearch={() => {}}
          onFilter={() => {}}
        />
      </Container>
      <MiniPlayer />
    </>
  );
};

export default Tracks;
