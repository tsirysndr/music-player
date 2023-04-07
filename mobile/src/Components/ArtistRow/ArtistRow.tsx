import React, {FC} from 'react';
import {Artist} from '../../Types';
import styled from '@emotion/native';
import SvgMic from '../Icons/Mic';
import {TouchableWithoutFeedback} from 'react-native';

const Container = styled.View`
  height: 100px;
  width: 100%;
  flex-direction: row;
  align-items: center;
  padding: 0 20px;
`;

const Cover = styled.Image`
  width: 60px;
  height: 60px;
  border-radius: 30px;
`;

const Name = styled.Text`
  color: #fff;
  font-family: 'Gilroy-Bold';
  font-size: 16px;
  margin-left: 15px;
`;

const NoArtistCover = styled.View`
  width: 60px;
  height: 60px;
  background-color: #161515;
  border-radius: 30px;
  align-items: center;
  justify-content: center;
`;

export type ArtistRowProps = {
  artist: Artist;
  onSelected: () => void;
};

const ArtistRow: FC<ArtistRowProps> = props => {
  const {artist, onSelected} = props;
  return (
    <TouchableWithoutFeedback onPress={() => onSelected()}>
      <Container>
        {artist.picture && <Cover source={{uri: artist.picture}} />}
        {!artist.picture && (
          <NoArtistCover>
            <SvgMic height={30} width={30} />
          </NoArtistCover>
        )}
        <Name>{artist.name}</Name>
      </Container>
    </TouchableWithoutFeedback>
  );
};

export default ArtistRow;
