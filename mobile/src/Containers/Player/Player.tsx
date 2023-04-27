import styled from '@emotion/native';
import React, {FC} from 'react';
import CurrentTrack from '../../Components/CurrentTrack';
import PlayerControls from '../../Components/PlayerControls';
import {useCover} from '../../Hooks/useCover';

const Background = styled.ImageBackground`
  flex: 1;
  position: absolute;
  top: 0;
  left: 0;
  bottom: 0;
  right: 0;
  width: 100%;
`;

const NoBackground = styled.View`
  flex: 1;
  position: absolute;
  top: 0;
  left: 0;
  bottom: 0;
  right: 0;
  width: 100%;
  background-color: #000;
`;

const DarkBackground = styled.View`
  flex: 1;
  position: absolute;
  top: 0;
  left: 0;
  bottom: 0;
  right: 0;
  background-color: #000000d1;
`;

const Header = styled.View`
  height: 100px;
`;

export type PlayerProps = {
  track: any;
};

const Player: FC<PlayerProps> = ({track}) => {
  const cover = useCover(track?.cover);
  return (
    <>
      {track?.cover && (
        <>
          <Background
            source={{
              uri: cover,
            }}
          />
          <DarkBackground />
        </>
      )}
      {!track?.cover && <NoBackground />}
      <Header />
      <CurrentTrack />
      <PlayerControls />
    </>
  );
};

export default Player;
