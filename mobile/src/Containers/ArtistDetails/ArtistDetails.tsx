import styled from '@emotion/native';
import React, {FC} from 'react';
import MiniPlayer from '../../Components/MiniPlayer';
import Header from '../../Components/Header';

const Container = styled.View`
  flex: 1;
`;

export type ArtistDetailsProps = {
  onGoBack: () => void;
};

const ArtistDetails: FC<ArtistDetailsProps> = props => {
  const {onGoBack} = props;
  return (
    <>
      <Container>
        <Header title={'Big K.R.I.T.'} onGoBack={onGoBack} />
      </Container>
      <MiniPlayer />
    </>
  );
};

export default ArtistDetails;
