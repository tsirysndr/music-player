import React, {FC} from 'react';
import styled from '@emotion/native';
import MiniPlayer from '../../Components/MiniPlayer';

const Container = styled.View`
  flex: 1;
  background-color: #000;
`;

const MiniPlayerWrapper = styled.View`
  background-color: #000;
`;

const Devices: FC = () => {
  return (
    <>
      <Container></Container>
      <MiniPlayerWrapper>
        <MiniPlayer />
      </MiniPlayerWrapper>
    </>
  );
};

export default Devices;
