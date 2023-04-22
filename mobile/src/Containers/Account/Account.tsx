import styled from '@emotion/native';
import React, {FC} from 'react';
import MiniPlayer from '../../Components/MiniPlayer';

const Container = styled.View`
  flex: 1;
  background-color: #000;
`;

const Title = styled.Text`
  color: #fff;
  font-family: 'Gilroy-Bold';
  font-size: 24px;
`;

const MiniPlayerWrapper = styled.View`
  background-color: #000;
`;

const Account: FC = () => {
  return (
    <>
      <Container>
        <Title>Account</Title>
      </Container>
      <MiniPlayerWrapper>
        <MiniPlayer />
      </MiniPlayerWrapper>
    </>
  );
};

export default Account;
