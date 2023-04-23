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

export type AlbumDetailsProps = {
  album: any;
  onGoBack: () => void;
};

const AlbumDetails: FC<AlbumDetailsProps> = props => {
  const {album, onGoBack} = props;
  return (
    <Container>
      <Container>
        <Title></Title>
      </Container>
      <MiniPlayer />
    </Container>
  );
};

export default AlbumDetails;
