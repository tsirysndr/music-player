import styled from '@emotion/native';
import React, {FC} from 'react';
import MiniPlayer from '../../Components/MiniPlayer';
import Header from '../../Components/Header';
import {Artist} from '../../Types';

const Container = styled.View`
  flex: 1;
`;

export type ArtistDetailsProps = {
  artist: Artist;
  onGoBack: () => void;
};

const ArtistDetails: FC<ArtistDetailsProps> = props => {
  const {artist, onGoBack} = props;
  return (
    <>
      <Container>
        <Header title={artist.name} onGoBack={onGoBack} />
      </Container>
      <MiniPlayer />
    </>
  );
};

export default ArtistDetails;
